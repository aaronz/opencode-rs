mod state;
mod pty;
mod cli;
mod diff;
mod dsl;

pub use state::StateTester;
pub use pty::PtySimulator;
pub use cli::CliTester;
pub use diff::BufferDiff;
pub use dsl::TestDsl;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(true);
    }
}
