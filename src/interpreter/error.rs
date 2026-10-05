use std::fmt;

/// An error found before or during execution, reported with its source position.
#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub line: usize,
    pub column: Option<usize>,
    pub message: String,
}

impl Error {
    pub fn new(line: usize, column: Option<usize>, message: impl Into<String>) -> Self {
        Error {
            line,
            column,
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.column {
            Some(column) => write!(f, "line {}, column {}: {}", self.line, column, self.message),
            None => write!(f, "line {}: {}", self.line, self.message),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_with_column() {
        let error = Error::new(3, Some(7), "unexpected `#`");
        assert_eq!(error.to_string(), "line 3, column 7: unexpected `#`");
    }

    #[test]
    fn display_without_column() {
        let error = Error::new(3, None, "missing `end if`");
        assert_eq!(error.to_string(), "line 3: missing `end if`");
    }
}
