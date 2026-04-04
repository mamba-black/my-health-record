use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::panic::Location;

pub enum ClickCareError {
   GenericError {
       message: String,
       file: &'static str,
       line: u32,
   },
}

impl ClickCareError {

    #[track_caller]
    pub fn generic<M: Into<String>>(msg: M) -> Self {
        let location = Location::caller();

        ClickCareError::GenericError {
            message: msg.into(),
            file: location.file(),
            line: location.line(),
        }
    }
}

impl Display for ClickCareError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClickCareError::GenericError { message, file, line } => {
                write!(f, "ClickCareError: {} (en {}:{})", message, file, line)
            }
        }
    }
}

impl Debug for ClickCareError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Error for ClickCareError {

}


#[cfg(test)]
mod test {
    use crate::domain::error::ClickCareError;

    #[test]
    fn test_error() -> Result<(), ClickCareError> {
        ClickCareError::generic("test");
        Ok(())
    }
}
