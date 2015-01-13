use std::io::{Reader, IoResult};

pub struct BitBuffer<T: Reader> {
  buffer: T,
  bit_position: u8,
  byte: u8
}

impl <T: Reader> Reader for BitBuffer<T> {
  fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
    for pos in (0..buf.len()){
      buf[pos] = try!(self.read_bits(8).map(|opt| opt as u8 ));
    }
    return Ok(buf.len())
  }
}

impl <T: Reader> BitBuffer<T> {
  pub fn new(data: T) -> BitBuffer<T> {
    BitBuffer{buffer: data, bit_position: 8, byte: 0}
  }

  // Reads bit one by one
  #[inline]
  pub fn read_bit(&mut self) -> IoResult<u8> {
    if self.bit_position == 8 {
      self.byte = try!(self.buffer.read_byte());
      self.bit_position = 0;
    }
    let bit = (self.byte >> (8 - self.bit_position - 1) as usize) & 0x1; //MSB
    self.bit_position += 1;
    Ok(bit)
  }

  // bigEndian MSB
  pub fn read_bits(&mut self, count: usize) -> IoResult<usize> {
    let mut word = 0us;
    for idx in (0..count) {
      let bit = try!(self.read_bit());
      word = word | ((bit as usize) << (count - 1 - idx));
    }
    Ok(word)
  }

  pub fn read_bits_reversed(&mut self, count: usize) -> IoResult<usize> {
    let mut word = 0us;
    for idx in (0..count) {
      let bit = try!(self.read_bit());
      word = word | ((bit as usize) << idx);
    }
    Ok(word)
  }
}

#[cfg(test)]
mod tests {
  use bitbuffer::BitBuffer;

  #[test]
  pub fn test_bit_buffer_read_bit(){
    let data = vec!(0b11001010, 0b11110000);
    println!("data: {:?}" ,data);
    let mut bb = BitBuffer::new(data.as_slice());
    let bits : Vec<u8> = (0..16).map(|_|{
      bb.read_bit().unwrap()
    }).collect();
    assert_eq!(bits, vec!(1,1,0,0,1,0,1,0, 1,1,1,1,0,0,0,0));
  }
  #[test]
  pub fn test_bit_buffer_read_bits_lsb_byte(){
    let data = vec!(0b11001010, 0b11110000);
    let mut bb = BitBuffer::new(data.as_slice());
    let bits = bb.read_bits_reversed(8).unwrap();
    assert_eq!(bits, 83);
  }
  #[test]
  pub fn test_bit_buffer_read_bits_lsb_less_then_byte(){
    let data = vec!(0b11001010, 0b11110000);
    let mut bb = BitBuffer::new(data.as_slice());
    let bits = bb.read_bits_reversed(7).unwrap();
    assert_eq!(bits, 83);
  }
  #[test]
  pub fn test_bit_buffer_read_bits_msb_byte(){
    let data = vec!(0b11001010, 0b11110000);
    let mut bb = BitBuffer::new(data.as_slice());
    let bits = bb.read_bits(8).unwrap();
    assert_eq!(bits, 202);
  }
  #[test]
  pub fn test_bit_buffer_read_bits_msb_less_then_byte(){
    let data = vec!(0b11001010, 0b11110000);
    let mut bb = BitBuffer::new(data.as_slice());
    let bits = bb.read_bits(7).unwrap();
    assert_eq!(bits, 101);
  }
}