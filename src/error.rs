use std::error::Error;

#[derive(Debug)]
pub struct ProgErr(Box<dyn Error>);
impl std::fmt::Display for ProgErr {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "Error:\n\t{}", self.0)
    }
}
impl std::error::Error for ProgErr {}

macro_rules! prog_err {
    ( $( $type: ty ),* ) => {
        $(
            impl std::convert::From<$type> for ProgErr {
                fn from(e: $type) -> Self {
                    Self(Box::from(e))
                }
            }
        )*
    };
}

prog_err!(glib::BoolError, crate::emulator::error::EmulatorError);
