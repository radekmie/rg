use crate::ist::Value;
use std::fmt::{Display, Formatter, Result};

impl<Id: Display + Ord> Display for Value<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Element { value } => write!(f, "{}", value),
            Self::Map { default, values } => {
                write!(f, "{{ :{}", default)?;
                for (key, value) in values.iter() {
                    write!(f, ", {}: {}", key, value)?;
                }
                write!(f, " }}")
            }
        }
    }
}
