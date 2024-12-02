use std::fmt::{Display, Formatter, Result};

pub fn write_with_separator<T: Display>(
    f: &mut Formatter<'_>,
    items: &[T],
    separator: &str,
) -> Result {
    let mut iter = items.iter();
    if let Some(item) = iter.next() {
        write!(f, "{item}")?;
        for item in iter {
            write!(f, "{separator}{item}")?;
        }
    }
    Ok(())
}
