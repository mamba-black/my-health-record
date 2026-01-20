use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum ClickCareError {
   GenericError {
       message: String,
       file: &'static str,
       line: u32,
   },
}

impl ClickCareError {
    pub fn generic<M: Into<String>>(msg: M) -> Self {
        ClickCareError::GenericError {
            message: msg.into(),
            file: file!(),
            line: line!(),
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

impl Error for ClickCareError {

}

