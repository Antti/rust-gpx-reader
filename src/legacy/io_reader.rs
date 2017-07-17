use std::io::Read;
use byteorder::{ReadBytesExt, LittleEndian};
use encoding::{Encoding, DecoderTrap};
use encoding::codec::singlebyte::SingleByteEncoding;

use super::super::{Error, ErrorKind, Result};

const DEFAULT_GPENCODING: &'static SingleByteEncoding = ::encoding::all::WINDOWS_1252;
const MAX_STRING_SIZE: usize = 65536;

pub trait IoReader: Read {
    fn skip(&mut self, n_bytes: i64) -> Result<()> {
        for _ in 0..n_bytes {
            self.read_byte()?;
        }
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8> {
        let buf = &mut [0u8];
        self.read_exact(buf)?;
        Ok(buf[0])
    }

    fn read_signed_byte(&mut self) -> Result<i8> {
        let buf = &mut [0u8];
        self.read_exact(buf)?;
        Ok(buf[0] as i8)
    }

    fn read_bytes(&mut self, n_bytes: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; n_bytes];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_bool(&mut self) -> Result<bool> {
        self.read_byte().map(|byte| byte > 0)
    }

    // Read 2 little-endian bytes as a short integer.
    fn read_short(&mut self) -> Result<i16> {
        self.read_i16::<LittleEndian>().map_err(Error::from)
    }

    // Read 4 little-endian bytes an integer.
    fn read_int(&mut self) -> Result<i32> {
        self.read_i32::<LittleEndian>().map_err(Error::from)
    }

    // Read 4 little-endian bytes as a float.
    fn read_float(&mut self) -> Result<f32> {
        self.read_f32::<LittleEndian>().map_err(Error::from)
    }

    // Read 8 little-endian bytes as a float.
    fn read_double(&mut self) -> Result<f64> {
        self.read_f64::<LittleEndian>().map_err(Error::from)
    }

    // Read length of the string stored in 1 byte and followed by character bytes.
    fn read_byte_sized_string(&mut self, size: usize) -> Result<String> {
        let len = self.read_byte()? as usize;
        self.read_string(size, Some(len))
    }

    // Read length of the string stored in 1 integer and followed by character bytes.
    fn read_int_sized_string(&mut self) -> Result<String> {
        let size = self.read_int()?;
        self.read_string(size as usize, None)
    }

    // Read length of the string increased by 1 and stored in 1 integer
    // followed by length of the string in 1 byte and finally followed by
    // character bytes.
    fn read_int_byte_sized_string(&mut self) -> Result<String> {
        let size = self.read_int()? as usize - 1;
        self.read_byte_sized_string(size)
    }

    // size is a number of bytes to read (always) and length, it's an optional string length,
    // which is used to truncate read string.
    fn read_string(&mut self, size: usize, length: Option<usize>) -> Result<String> {
        debug!("Reading string size:{size}, length:{length:?}",
               size = size,
               length = length);
        if size > MAX_STRING_SIZE {
            return Err(ErrorKind::FormatError(format!("Requested to read {} bytes string, too much...", size)).into());
        }
        let need_to_read = match length {
            None => size,
            Some(_) if size > 0 => size,
            Some(length) => length,
        };
        if let Some(len) = length {
            if len > need_to_read {
                return Err(ErrorKind::FormatError(format!("Requested to return {} bytes, but will read only {} (len > size)",
                                                          len,
                                                          need_to_read))
                                   .into());
            }
        }
        let mut buf: Vec<u8> = vec![0u8; need_to_read];
        let read_count = self.read(&mut buf)?;
        if read_count < need_to_read {
            return Err(ErrorKind::FormatError(format!("Read {} bytes, expected {}", read_count, need_to_read)).into());
        }
        let truncated_buf = match length {
            Some(len) => &buf[0..len],
            None => &buf as &[u8],
        };
        let s = convert_to_string(truncated_buf)?;
        debug!("read string: {:?}", s);
        Ok(s)
    }
}

#[cfg(not(feature = "autodetect_encoding"))]
fn convert_to_string(buf: &[u8]) -> Result<String> {
    DEFAULT_GPENCODING.decode(buf, DecoderTrap::Replace).map_err(|_| ErrorKind::EncodingError.into())
}

#[cfg(feature = "autodetect_encoding")]
fn convert_to_string(buf: &[u8]) -> Result<String> {
    match &::uchardet::detect_encoding_name(buf)?.unwrap_or("DEFAULT".to_string()) as &str {
            "windows-1251" => ::encoding::all::WINDOWS_1251.decode(buf, DecoderTrap::Replace),
            "windows-1252" => ::encoding::all::WINDOWS_1252.decode(buf, DecoderTrap::Replace),
            "UTF-8" => ::encoding::all::UTF_8.decode(buf, DecoderTrap::Replace),
            "ISO-8859-7" => ::encoding::all::ISO_8859_7.decode(buf, DecoderTrap::Replace),
            "KOI8-R" => ::encoding::all::WINDOWS_1251.decode(buf, DecoderTrap::Replace), // It's probably 1251 anyway
            "x-mac-cyrillic" => ::encoding::all::WINDOWS_1251.decode(buf, DecoderTrap::Replace), // It's probably 1251 anyway
            "windows-1255" => ::encoding::all::WINDOWS_1251.decode(buf, DecoderTrap::Replace), // It's probably 1251 anyway
            "ISO-8859-8" => ::encoding::all::WINDOWS_1251.decode(buf, DecoderTrap::Replace), // It's probably 1251 anyway
            "DEFAULT" => DEFAULT_GPENCODING.decode(buf, DecoderTrap::Replace), // Error detecting, probably not enough data
            enc => {
                println!("Detected unhandled encoding: {}", enc);
                DEFAULT_GPENCODING.decode(buf, DecoderTrap::Replace)
            }
            // None =>
        }
        .map_err(Error::from)
}

impl<T: Read> IoReader for T {}

#[cfg(test)]
mod tests {
    use super::IoReader;

    #[test]
    pub fn test_io_reader_read_byte() {
        let data: &[u8] = &[0b11001010, 0b11110000];
        println!("data: {:?}", data);
        let mut io = data;
        let bits: Vec<u8> = (0..2).map(|_| io.read_byte().unwrap()).collect();
        assert_eq!(&bits as &[u8], data);
    }
    #[test]
    pub fn test_io_reader_skip() {
        let data: &[u8] = &[0b11001010, 0b11110000];
        println!("data: {:?}", data);
        let mut io = data;
        io.skip(1).unwrap();
        assert_eq!(io.read_byte().unwrap(), data[1]);
    }
}
