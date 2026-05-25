/// Crate-level error type for the `sources` crate.
#[derive(Debug, thiserror::Error)]
pub enum SourcesError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
