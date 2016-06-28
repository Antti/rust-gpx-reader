use std::io;
use std::convert::From;
use std::error;
use std::fmt;


#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    StringEncodingError(String),
    FormatError(String)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Tab read error: {}", error::Error::description(self))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::IoError(_) => "IoError",
            &Error::StringEncodingError(_) => "String Encoding Error",
            &Error::FormatError(_) => "File format Error"
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &Error::IoError(ref err) => Some(err),
            &Error::StringEncodingError(_) => None,
            &Error::FormatError(_) => None
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl <'a> From<::std::borrow::Cow<'a, str>> for Error {
    fn from(err: ::std::borrow::Cow<'a, str>) -> Error {
        Error::StringEncodingError(err.to_string())
    }
}

#[cfg(feature = "autodetect_encoding")]
impl From<::uchardet::EncodingDetectorError> for Error {
    fn from(err: ::uchardet::EncodingDetectorError) -> Error {
        Error::StringEncodingError(err.to_string())
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err)
    }
}
