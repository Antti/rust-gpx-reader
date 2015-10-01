#[macro_use]
extern crate log;
extern crate env_logger;
extern crate byteorder;
extern crate encoding;

pub mod gpx;
pub mod legacy;
pub mod error;
mod bitbuffer;

use std::result;

pub type Result<T> = result::Result<T, error::Error>;
