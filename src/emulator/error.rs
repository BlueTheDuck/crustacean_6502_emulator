use std::boxed::Box;
use std::error::Error;
use std::fmt::{Display, Error as FmtError, Formatter};

#[derive(Debug)]
pub enum CpuError {
    UnknownOp(u8),
    Break,
    Suberror(Box<dyn Error>),
}

impl Display for CpuError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "Emulator Error: {}",
            match self {
                CpuError::Suberror(e) => e.description().to_string(),
                CpuError::Break => "Emulator Terminated".to_string(),
                CpuError::UnknownOp(code) => format!("Unknown OP with code {:02X}", code),
            }
        )
    }
}
impl Error for CpuError {}
unsafe impl std::marker::Send for CpuError {}
unsafe impl std::marker::Sync for CpuError {}
