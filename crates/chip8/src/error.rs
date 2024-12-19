use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Invalid opcode {0:#06x}")]
    InvalidOpcode(u16),

    #[error("Memory access error {0:#06x}")]
    MemoryAccessError(u16),
}
