mod cli;
mod dialog_tester;
mod diff;
mod dsl;
mod pty;
mod snapshot;
mod state;

pub use cli::{CliOutput, CliTester};
pub use dialog_tester::{assert_render_result, DialogRenderTester};
pub use diff::{BufferDiff, CellDiff, DiffResult, IgnoreOptions};
pub use dsl::{TestDsl, WaitPredicate};
pub use pty::PtySimulator;
pub use snapshot::{load_snapshot, save_snapshot};
pub use state::{DiffType, StateDiff, StateDiffEntry, StateSnapshot, StateTester, TerminalState};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // Basic test to verify the library compiles
    }
}
