mod cli;
mod diff;
mod dsl;
mod pty;
mod state;

pub use cli::CliTester;
pub use diff::BufferDiff;
pub use dsl::TestDsl;
pub use pty::PtySimulator;
pub use state::StateTester;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(true);
    }
}
