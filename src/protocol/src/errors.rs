use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub enum ParseError {
    NotValidHeaderLength,
    NotValidFieldLength,
    NotValidFieldDataLength,
    NotValidFieldsCount,
    NotValidMethod,
    NotValidFieldType,
    DuplicateFieldFound,
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Error {
        match error {
            ParseError::NotValidHeaderLength => Error::new(ErrorKind::Other, "Invalid header length"),
            ParseError::NotValidFieldLength => Error::new(ErrorKind::Other, "Invalid field length"),
            ParseError::NotValidFieldDataLength => Error::new(ErrorKind::Other, "Invalid field data length"),
            ParseError::NotValidFieldsCount => Error::new(ErrorKind::Other, "Invalid fields count"),
            ParseError::NotValidMethod => Error::new(ErrorKind::Other, "Invalid method"),
            ParseError::NotValidFieldType => Error::new(ErrorKind::Other, "Invalid field type"),
            ParseError::DuplicateFieldFound => Error::new(ErrorKind::Other, "Duplicate field found"),
        }
    }
}