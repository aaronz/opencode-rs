use clap::Args;
use std::io::{self, Write};

#[derive(Args, Debug)]
pub(crate) struct CompletionArgs {
    #[arg(value_enum, default_value = "bash")]
    pub shell: Shell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Pwsh,
}

impl std::str::FromStr for Shell {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            "powershell" | "pwsh" => Ok(Shell::Pwsh),
            _ => Err(format!(
                "Unknown shell: {}. Use bash, zsh, fish, or powershell.",
                s
            )),
        }
    }
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shell::Bash => write!(f, "bash"),
            Shell::Zsh => write!(f, "zsh"),
            Shell::Fish => write!(f, "fish"),
            Shell::Pwsh => write!(f, "powershell"),
        }
    }
}

const SUBCOMMANDS: &[&str] = &[
    "run",
    "serve",
    "desktop",
    "account",
    "config",
    "agent",
    "bash",
    "completion",
    "models",
    "providers",
    "mcp",
    "session",
    "list",
    "stats",
    "terminal",
    "db",
    "git-hub",
    "git-lab",
    "pr",
    "export",
    "import",
    "generate",
    "web",
    "thread",
    "join",
    "uninstall",
    "upgrade",
    "debug",
    "acp",
    "workspace-serve",
    "palette",
    "plugin",
    "permissions",
    "shortcuts",
    "workspace",
    "ui",
    "project",
    "files",
    "prompt",
    "quick",
    "tui",
];

pub(crate) fn run(args: CompletionArgs) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    match args.shell {
        Shell::Bash => {
            write_bash_completions(&mut handle).unwrap();
        }
        Shell::Zsh => {
            write_zsh_completions(&mut handle).unwrap();
        }
        Shell::Fish => {
            write_fish_completions(&mut handle).unwrap();
        }
        Shell::Pwsh => {
            write_powershell_completions(&mut handle).unwrap();
        }
    }
}

fn write_bash_completions<W: Write>(w: &mut W) -> io::Result<()> {
    writeln!(w, "# opencode-rs bash completion")?;
    writeln!(w, "_opencode_rs() {{")?;
    writeln!(w, "    local cur prev opts")?;
    writeln!(w, "    COMPREPLY=()")?;
    writeln!(w, "    cur=\"${{COMP_WORDS[COMP_CWORD]}}\"")?;
    writeln!(w, "    prev=\"${{COMP_WORDS[COMP_CWORD-1]}}\"")?;
    writeln!(w)?;
    writeln!(w, "    opts=\"{}\"", SUBCOMMANDS.join(" "))?;
    writeln!(w)?;
    writeln!(w, "    if [[ ${{cur}} == * ]]; then")?;
    writeln!(
        w,
        "        COMPREPLY=($(compgen -W \"${{opts}}\" -- \"${{cur}}\"))"
    )?;
    writeln!(w, "        return 0")?;
    writeln!(w, "    fi")?;
    writeln!(w, "}}")?;
    writeln!(w)?;
    writeln!(w, "complete -F _opencode_rs opencode-rs")?;
    writeln!(w)?;
    writeln!(w, "# Setup for 'opencode' alias if it exists")?;
    writeln!(w, "complete -F _opencode_rs opencode 2>/dev/null || true")?;
    Ok(())
}

fn write_zsh_completions<W: Write>(w: &mut W) -> io::Result<()> {
    writeln!(w, "#compdef opencode-rs opencode")?;
    writeln!(w)?;
    writeln!(w, "_opencode_rs() {{")?;
    writeln!(w, "    local -a commands")?;
    writeln!(w, "    commands=(")?;
    for cmd in SUBCOMMANDS {
        writeln!(w, "        \"{cmd}:\"")?;
    }
    writeln!(w, "    )")?;
    writeln!(w)?;
    writeln!(w, "    _describe 'command' commands")?;
    writeln!(w, "}}")?;
    writeln!(w)?;
    writeln!(w, "_compdef _opencode_rs opencode-rs")?;
    writeln!(w, "compdef _opencode_rs opencode")?;
    Ok(())
}

fn write_fish_completions<W: Write>(w: &mut W) -> io::Result<()> {
    writeln!(w, "# opencode-rs fish completion")?;
    writeln!(w)?;
    writeln!(
        w,
        "complete -c opencode-rs -f -a '{}'",
        SUBCOMMANDS.join("' '")
    )?;
    writeln!(w)?;
    writeln!(
        w,
        "complete -c opencode -f -a '{}' 2>/dev/null || true",
        SUBCOMMANDS.join("' '")
    )?;
    Ok(())
}

fn write_powershell_completions<W: Write>(w: &mut W) -> io::Result<()> {
    writeln!(w, "# opencode-rs PowerShell completion")?;
    writeln!(w)?;
    writeln!(w, "$script:OpenCodeCommands = @(")?;
    for cmd in SUBCOMMANDS {
        writeln!(w, "    '{}'", cmd)?;
    }
    writeln!(w, ")")?;
    writeln!(w)?;
    writeln!(
        w,
        "Register-ArgumentCompleter -Native -CommandName opencode-rs -ScriptBlock {{"
    )?;
    writeln!(
        w,
        "    param($wordToComplete, $commandAst, $cursorPosition)"
    )?;
    writeln!(w, "    $command = $commandAst.CommandElements[0].Value")?;
    writeln!(w, "    if ($command -eq 'opencode-rs') {{")?;
    writeln!(w, "        $commands = $script:OpenCodeCommands")?;
    writeln!(w, "        $commands | Where-Object {{ $_ -like \"$wordToComplete*\" }} | ForEach-Object {{ $_ }}")?;
    writeln!(w, "    }}")?;
    writeln!(w, "}}")?;
    writeln!(w)?;
    writeln!(
        w,
        "Register-ArgumentCompleter -Native -CommandName opencode -ScriptBlock {{"
    )?;
    writeln!(
        w,
        "    param($wordToComplete, $commandAst, $cursorPosition)"
    )?;
    writeln!(w, "    $command = $commandAst.CommandElements[0].Value")?;
    writeln!(w, "    if ($command -eq 'opencode') {{")?;
    writeln!(w, "        $commands = $script:OpenCodeCommands")?;
    writeln!(w, "        $commands | Where-Object {{ $_ -like \"$wordToComplete*\" }} | ForEach-Object {{ $_ }}")?;
    writeln!(w, "    }}")?;
    writeln!(w, "}}")?;
    Ok(())
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_from_str_bash() {
        let shell: Shell = "bash".parse().unwrap();
        assert_eq!(shell, Shell::Bash);
    }

    #[test]
    fn test_shell_from_str_zsh() {
        let shell: Shell = "zsh".parse().unwrap();
        assert_eq!(shell, Shell::Zsh);
    }

    #[test]
    fn test_shell_from_str_fish() {
        let shell: Shell = "fish".parse().unwrap();
        assert_eq!(shell, Shell::Fish);
    }

    #[test]
    fn test_shell_from_str_powershell() {
        let shell: Shell = "powershell".parse().unwrap();
        assert_eq!(shell, Shell::Pwsh);
    }

    #[test]
    fn test_shell_from_str_pwsh() {
        let shell: Shell = "pwsh".parse().unwrap();
        assert_eq!(shell, Shell::Pwsh);
    }

    #[test]
    fn test_shell_from_str_unknown() {
        let result: Result<Shell, _> = "csh".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_display() {
        assert_eq!(Shell::Bash.to_string(), "bash");
        assert_eq!(Shell::Zsh.to_string(), "zsh");
        assert_eq!(Shell::Fish.to_string(), "fish");
        assert_eq!(Shell::Pwsh.to_string(), "powershell");
    }

    #[test]
    fn test_all_shells_generate_output() {
        let shells = [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Pwsh];

        for shell in shells {
            let mut output = Vec::new();
            let args = CompletionArgs { shell };
            match args.shell {
                Shell::Bash => write_bash_completions(&mut output).unwrap(),
                Shell::Zsh => write_zsh_completions(&mut output).unwrap(),
                Shell::Fish => write_fish_completions(&mut output).unwrap(),
                Shell::Pwsh => write_powershell_completions(&mut output).unwrap(),
            }
            let output_str = String::from_utf8(output).unwrap();
            assert!(
                !output_str.is_empty(),
                "Shell {} should generate output",
                shell
            );
            assert!(
                output_str.contains("opencode"),
                "Shell {} should mention opencode",
                shell
            );
        }
    }

    #[test]
    fn test_bash_completions_contains_commands() {
        let mut output = Vec::new();
        write_bash_completions(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("_opencode_rs"));
        assert!(output_str.contains("complete -F _opencode_rs opencode-rs"));
        for cmd in SUBCOMMANDS {
            assert!(
                output_str.contains(cmd),
                "Bash completions should contain {}",
                cmd
            );
        }
    }

    #[test]
    fn test_zsh_completions_contains_commands() {
        let mut output = Vec::new();
        write_zsh_completions(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("_opencode_rs"));
        assert!(output_str.contains("_compdef _opencode_rs opencode-rs"));
        for cmd in SUBCOMMANDS {
            assert!(
                output_str.contains(cmd),
                "Zsh completions should contain {}",
                cmd
            );
        }
    }

    #[test]
    fn test_fish_completions_contains_commands() {
        let mut output = Vec::new();
        write_fish_completions(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("complete -c opencode-rs"));
        for cmd in SUBCOMMANDS {
            assert!(
                output_str.contains(cmd),
                "Fish completions should contain {}",
                cmd
            );
        }
    }

    #[test]
    fn test_powershell_completions_contains_commands() {
        let mut output = Vec::new();
        write_powershell_completions(&mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("$script:OpenCodeCommands"));
        assert!(output_str.contains("Register-ArgumentCompleter"));
        for cmd in SUBCOMMANDS {
            assert!(
                output_str.contains(cmd),
                "PowerShell completions should contain {}",
                cmd
            );
        }
    }
}
