use std::boxed::Box;
use std::error::Error;
use std::fmt::{Display, Error as FmtError, Formatter};

#[derive(Debug)]
pub enum EmulatorError {
    UnknownOp(u8),
    Break,
    Suberror(Box<dyn Error>),
}

impl Display for EmulatorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "Emulator Error: {}",
            match self {
                EmulatorError::Suberror(e) => e.description().to_string(),
                EmulatorError::Break => "Emulator Terminated".to_string(),
                EmulatorError::UnknownOp(code) => format!("Unknown OP with code {:02X}", code),
            }
        )
    }
}
impl Error for EmulatorError {}
