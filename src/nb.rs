use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NbError {
    /// Error type for when `nb` cannot be executed for any reason
    #[error("could not execute nb")]
    ExecutionFailure(#[from] io::Error),

    /// Error type for when `nb` itself fails
    #[error("nb: {0}")]
    NbFailure(String),
}
