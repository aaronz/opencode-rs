pub mod config;
pub mod diff;
pub mod dsl;
pub mod state;

#[cfg(feature = "ratatui")]
pub mod backend {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    pub struct TestTerminal {
        terminal: Terminal<TestBackend>,
    }

    impl TestTerminal {
        pub fn new(width: u16, height: u16) -> Self {
            let backend = TestBackend::new(width, height);
            let terminal = Terminal::new(backend).unwrap();
            Self { terminal }
        }

        pub fn default_size() -> Self {
            Self::new(80, 24)
        }

        pub fn buffer(&self) -> &ratatui::buffer::Buffer {
            self.terminal.backend().buffer()
        }
    }
}

#[cfg(feature = "pty")]
pub mod pty {
    use portable_pty::{native_pty_system, Child, CommandBuilder, PtyPair, PtySize};
    use std::io::{Read, Write};
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    pub struct PtySession {
        pair: PtyPair,
        child: Box<dyn Child + Send + Sync>,
        writer: Mutex<Option<Box<dyn Write + Send>>>,
    }

    impl PtySession {
        pub fn new(width: u16, height: u16) -> anyhow::Result<Self> {
            let pty_system = native_pty_system();
            let pair = pty_system.openpty(PtySize {
                rows: height,
                cols: width,
                pixel_width: 0,
                pixel_height: 0,
            })?;
            let child = pair.slave.spawn_command(CommandBuilder::new("true"))?;
            let writer = pair.master.take_writer()?;
            Ok(Self {
                pair,
                child,
                writer: Mutex::new(Some(writer)),
            })
        }

        pub fn spawn_command(program: &str, args: &[&str]) -> anyhow::Result<Self> {
            let pty_system = native_pty_system();
            let pair = pty_system.openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })?;
            let mut cmd = CommandBuilder::new(program);
            for arg in args {
                cmd.arg(arg);
            }
            let child = pair.slave.spawn_command(cmd)?;
            let writer = pair.master.take_writer()?;
            Ok(Self {
                pair,
                child,
                writer: Mutex::new(Some(writer)),
            })
        }

        pub fn write(&self, data: &[u8]) -> anyhow::Result<()> {
            let mut guard = self.writer.lock().unwrap();
            if let Some(ref mut writer) = *guard {
                writer.write_all(data)?;
                writer.flush()?;
                Ok(())
            } else {
                anyhow::bail!("PT writer already taken")
            }
        }

        pub fn write_str(&self, s: &str) -> anyhow::Result<()> {
            self.write(s.as_bytes())
        }

        pub fn read(&mut self, timeout: Duration) -> anyhow::Result<String> {
            let mut reader = self.pair.master.try_clone_reader()?;
            let mut output = String::new();
            let start = Instant::now();
            let poll_interval = Duration::from_millis(50);

            while start.elapsed() < timeout {
                let mut buf = [0u8; 4096];
                match reader.read(&mut buf) {
                    Ok(0) => {
                        if start.elapsed() > Duration::from_millis(200) && !output.is_empty() {
                            break;
                        }
                    }
                    Ok(n) => {
                        output.push_str(std::str::from_utf8(&buf[..n]).unwrap_or(""));
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(_) => break,
                }
                std::thread::sleep(poll_interval);
            }
            Ok(output)
        }

        pub fn wait(&mut self) -> anyhow::Result<u32> {
            let status = self.child.wait()?;
            Ok(status.exit_code())
        }

        pub fn expect(&mut self, pattern: &str, timeout: Duration) -> anyhow::Result<bool> {
            let start = Instant::now();
            let poll_interval = Duration::from_millis(20);

            while start.elapsed() < timeout {
                let mut tmp = [0u8; 1024];
                match self.pair.master.try_clone_reader()?.read(&mut tmp) {
                    Ok(0) => {
                        if start.elapsed() > Duration::from_millis(50) {
                            break;
                        }
                    }
                    Ok(n) => {
                        let s = std::str::from_utf8(&tmp[..n]).unwrap_or("");
                        if s.contains(pattern) {
                            return Ok(true);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(_) => break,
                }
                std::thread::sleep(poll_interval);
            }
            Ok(false)
        }

        pub fn contains(&mut self, expected: &str, timeout: Duration) -> anyhow::Result<bool> {
            self.expect(expected, timeout)
        }
    }

    pub fn send_key(key: &str) -> Vec<u8> {
        match key {
            "enter" => vec![b'\n'],
            "escape" => vec![b'\x1b'],
            "tab" => vec![b'\t'],
            "up" => b"\x1b[A".to_vec(),
            "down" => b"\x1b[B".to_vec(),
            "right" => b"\x1b[C".to_vec(),
            "left" => b"\x1b[D".to_vec(),
            _ => key.as_bytes().to_vec(),
        }
    }
}

#[cfg(feature = "cli")]
pub mod cli {
    use assert_cmd::Command;
    use std::io::{BufRead, BufReader};
    use std::process::{Child, Stdio};

    pub struct CliTester {
        cmd: Command,
    }

    impl CliTester {
        pub fn cargo_bin(name: &str) -> anyhow::Result<Self> {
            Ok(Self {
                cmd: Command::cargo_bin(name)?,
            })
        }
        pub fn new(program: &str) -> Self {
            Self {
                cmd: Command::new(program),
            }
        }
        pub fn arg(&mut self, arg: &str) -> &mut Self {
            self.cmd.arg(arg);
            self
        }
        pub fn args(&mut self, args: &[&str]) -> &mut Self {
            self.cmd.args(args);
            self
        }
        pub fn write_stdin(&mut self, input: &str) -> &mut Self {
            self.cmd.write_stdin(input);
            self
        }
        pub fn assert(&mut self) -> assert_cmd::assert::Assert {
            self.cmd.assert()
        }
    }

    pub struct InteractiveCli {
        child: Child,
    }

    impl InteractiveCli {
        pub fn new(program: &str) -> anyhow::Result<Self> {
            let mut child = std::process::Command::new(program)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            Ok(Self { child })
        }

        pub fn send_line(&mut self, input: &str) -> anyhow::Result<()> {
            use std::io::Write;
            if let Some(stdin) = self.child.stdin.as_mut() {
                writeln!(stdin, "{}", input)?;
            }
            Ok(())
        }

        pub fn read_line(&mut self) -> anyhow::Result<String> {
            use std::io::Read;
            if let Some(stdout) = self.child.stdout.as_mut() {
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                reader.read_line(&mut line)?;
                Ok(line)
            } else {
                Ok(String::new())
            }
        }

        pub fn interact<F>(&mut self, mut handler: F) -> anyhow::Result<()>
        where
            F: FnMut(&str) -> Option<String>,
        {
            use std::io::{Read, Write};

            loop {
                if let Some(stdout) = self.child.stdout.as_mut() {
                    let mut buf = [0u8; 1024];
                    match stdout.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let output = String::from_utf8_lossy(&buf[..n]);
                            if let Some(response) = handler(&output) {
                                if let Some(stdin) = self.child.stdin.as_mut() {
                                    writeln!(stdin, "{}", response)?;
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
            Ok(())
        }

        pub fn wait(&mut self) -> anyhow::Result<u32> {
            let status = self.child.wait()?;
            Ok(status.code().unwrap_or(-1) as u32)
        }
    }

    pub fn assert_contains(output: &str, expected: &str) -> bool {
        output.contains(expected)
    }

    pub fn assert_starts_with(output: &str, prefix: &str) -> bool {
        output.starts_with(prefix)
    }

    pub fn assert_ends_with(output: &str, suffix: &str) -> bool {
        output.ends_with(suffix)
    }

    pub fn assert_matches(output: &str, pattern: &str) -> bool {
        regex::Regex::new(pattern)
            .map(|re| re.is_match(output))
            .unwrap_or(false)
    }
}

#[cfg(feature = "ratatui")]
pub use backend::TestTerminal;

#[cfg(feature = "ratatui")]
pub use state::{char_key, enter, escape, AppEvent, AppState, StateTester};

#[cfg(feature = "pty")]
pub use pty::PtySession;

#[cfg(feature = "cli")]
pub use cli::{CliTester, InteractiveCli};

pub use dsl::{tui, TuiTestBuilder};

pub use diff::{BufferDiff, DiffCell, DiffResult, DiffStats};

pub use config::{is_test_mode, TestModeConfig as TestConfig};
