use super::Error;
use std::fmt::{Display, Formatter, Result};

impl<Id: Display> Display for Error<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Todo(identifier) => {
                write!(f, "TODO({identifier})")
            }
        }
    }
}
