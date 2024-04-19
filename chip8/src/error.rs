use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Invalid opcode: {0:#06x}")]
    InvalidOpcode(u16),
}

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("Couldn't load file or directory at {path:?})")]
    InvalidPath { path: PathBuf },
    #[error("Couldn't read file or directory at {path:?}")]
    ReadError { path: PathBuf },
    #[error("ROM is too big to fit in memory: {length:?} bytes")]
    TooLarge { length: usize },
}
