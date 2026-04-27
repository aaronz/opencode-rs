use std::path::PathBuf;

pub(crate) const DEFAULT_MAX_FILE_SIZE: usize = 100 * 1024;
pub(crate) const DEFAULT_MAX_TOTAL_SIZE: usize = 20 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum InstructionsError {
    #[error("File not found: {0}")]
    NotFound(PathBuf),
    #[error("File too large: {0} ({1} bytes, max {2})")]
    FileTooLarge(PathBuf, usize, usize),
    #[error("Total instructions too large: {0} bytes, max {1}")]
    TotalTooLarge(usize, usize),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Glob pattern error: {0}")]
    GlobError(String),
}