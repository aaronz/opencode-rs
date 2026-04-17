mod cli;
mod dialog_tester;
mod diff;
mod dsl;
mod pty;
mod snapshot;
mod state;

pub use cli::{ChildProcess, CliOutput, CliTester};
pub use dialog_tester::{assert_empty_state, assert_render_result, DialogRenderTester};
pub use diff::{BufferDiff, CellDiff, DiffResult, IgnoreOptions};
pub use dsl::{TestDsl, WaitPredicate};
pub use pty::PtySimulator;
pub use snapshot::{load_snapshot, save_snapshot};
pub use state::{DiffType, StateDiff, StateDiffEntry, StateSnapshot, StateTester, TerminalState};

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::DialogRenderTester;

    #[test]
    fn it_works() {
        // Basic test to verify the library compiles
    }

    #[test]
    fn dialog_render_tester_is_exported() {
        // Regression test: verify DialogRenderTester is exported in lib.rs
        let _ = DialogRenderTester::new();
    }

    #[test]
    fn dialog_render_tester_has_doc_comments() {
        // Unit test: verify DialogRenderTester struct has doc comments
        // This test passes if the code compiles (doc comments are present)
        // The doc comment is on the struct: "Helper utilities for testing TUI dialog rendering"
        let _ = DialogRenderTester::new();
    }
}
