use std::io::Read;
use byteorder::{ReadBytesExt, LittleEndian};
use encoding::{Encoding, DecoderTrap};
use encoding::codec::singlebyte::SingleByteEncoding;

use super::super::Result;
use super::super::error::Error;

const GPENCODING: &'static SingleByteEncoding  = ::encoding::all::WINDOWS_1251;

pub trait IoReader: Read {
    fn skip(&mut self, n_bytes: i64) -> Result<()> {
        for _ in (0..n_bytes) {
            try!(self.read_byte());
        }
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8> {
        let buf = &mut [0u8];
        try!(self.read(buf));
        Ok(buf[0])
    }

    fn read_signed_byte(&mut self) -> Result<i8> {
        let buf = &mut [0u8];
        try!(self.read(buf));
        Ok(buf[0] as i8)
    }

    fn read_bytes(&mut self, n_bytes: usize) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(n_bytes);
        try!(self.read(&mut buf));
        Ok(buf)
    }

    fn read_bool(&mut self) -> Result<bool> {
        self.read_byte().map(|byte| byte > 0)
    }

    // Read 2 little-endian bytes as a short integer.
    fn read_short(&mut self) -> Result<i16> {
        self.read_i16::<LittleEndian>().map_err(|err| Error::from(err))
    }

    // Read 4 little-endian bytes an integer.
    fn read_int(&mut self) -> Result<i32> {
        self.read_i32::<LittleEndian>().map_err(|err| Error::from(err))
    }

    // Read 4 little-endian bytes as a float.
    fn read_float(&mut self) -> Result<f32> {
        self.read_f32::<LittleEndian>().map_err(|err| Error::from(err))
    }

    // Read 8 little-endian bytes as a float.
    fn read_double(&mut self) -> Result<f64> {
        self.read_f64::<LittleEndian>().map_err(|err| Error::from(err))
    }

    // Read length of the string stored in 1 byte and followed by character bytes.
    fn read_byte_sized_string(&mut self, size: usize) -> Result<String> {
        let len = try!(self.read_byte()) as usize;
        self.read_string(size, Some(len))
    }

    // Read length of the string stored in 1 integer and followed by character bytes.
    fn read_int_sized_string(&mut self) -> Result<String> {
        let size = try!(self.read_int());
        self.read_string(size as usize, None)
    }

    // Read length of the string increased by 1 and stored in 1 integer
    // followed by length of the string in 1 byte and finally followed by
    // character bytes.
    fn read_int_byte_sized_string(&mut self) -> Result<String> {
        let size = try!(self.read_int()) as usize - 1;
        return self.read_byte_sized_string(size)
    }

    // size is a number of bytes to read (always) and lenght, it's an optional string length,
    // which is used to truncate read string.
    fn read_string(&mut self, size: usize, length: Option<usize>) -> Result<String> {
        debug!("Reading size:{size}, length:{length:?}", size=size, length=length);
        let need_to_read = match length {
            None => size,
            Some(_) if size > 0 => size,
            Some(length) => length
        };
        if let Some(len) = length {
            if len > need_to_read {
                return Err(Error::FormatError(format!("Requested to return {} bytes, but will read only {}", len, need_to_read)));
            }
        }
        let mut buf : Vec<u8> = vec![0u8; need_to_read];
        let read_count = try!(self.read(&mut buf));
        if read_count < need_to_read {
            return Err(Error::FormatError(format!("Read {} bytes, expected {}", read_count, need_to_read)));
        }
        let truncated_buf = match length {
            Some(len) => &buf[0..len],
            None => &buf as &[u8]
        };
        let s = try!(GPENCODING.decode(truncated_buf, DecoderTrap::Replace));
        Ok(s)
    }
}

impl <T: Read> IoReader for T {}

#[cfg(test)]
mod tests {
    use super::IoReader;

    #[test]
    pub fn test_io_reader_read_byte(){
        let data : &[u8] = &[0b11001010, 0b11110000];
        println!("data: {:?}", data);
        let mut io = data;
        let bits : Vec<u8> = (0..2).map(|_|{
            io.read_byte().unwrap()
        }).collect();
        assert_eq!(&bits as &[u8], data);
    }
    #[test]
    pub fn test_io_reader_skip(){
        let data : &[u8] = &[0b11001010, 0b11110000];
        println!("data: {:?}", data);
        let mut io = data;
        io.skip(1).unwrap();
        assert_eq!(io.read_byte().unwrap(), data[1]);
    }
}
