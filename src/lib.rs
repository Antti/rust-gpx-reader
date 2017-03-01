// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate byteorder;
extern crate encoding;

#[cfg(feature = "autodetect_encoding")]
extern crate uchardet;
#[macro_use]
extern crate error_chain;

pub mod gpx;
pub mod legacy;
pub mod error;
mod bitbuffer;

pub use error::{Error, ErrorKind, Result};
