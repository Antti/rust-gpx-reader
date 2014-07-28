pub struct BitBuffer {
  buffer: Box<Reader>,
  bit_position: u8,
  byte: u8
}

impl BitBuffer {
  pub fn new(data: Box<Reader>) -> BitBuffer {
    BitBuffer{buffer: data, bit_position: 8, byte: 0}
  }

  // Reads bit one by one
  #[inline]
  pub fn read_bit(&mut self) -> Option<u8> {
    if self.bit_position == 8 {
      let byte = self.buffer.read_byte();
      match byte {
        Ok(byte) => self.byte = byte,
        Err(_) => return None
      }
      self.bit_position = 0;
    }
    let bit = (self.byte >> (8 - self.bit_position - 1) as uint) & 0x1; //MSB
    self.bit_position += 1;
    Some(bit)
  }

  // bigEndian MSB
  pub fn read_bits(&mut self, count: uint) -> Option<uint> {
    let mut word = 0u;
    for idx in range(0, count) {
      match self.read_bit() {
        Some(bit) => { word = word | (bit as uint << (count - 1 - idx)) },
        None => return None
      }
    }
    Some(word)
  }

  pub fn read_bits_reversed(&mut self, count: uint) -> Option<uint> {
    let mut word = 0u;
    for idx in range(0, count) {
      match self.read_bit() {
        Some(bit) => { word = word | (bit as uint << idx) },
        None => return None
      }
    }
    Some(word)
  }


  pub fn read_byte(&mut self) -> Option<u8> {
    self.read_bits(8).map(|opt| opt as u8 )
  }

  pub fn read_le_i32(&mut self) -> Option<uint> {
    self.buffer.read_le_i32().ok().map(|x| x as uint)
  }
}

#[cfg(test)]
mod tests {
  use std::io::MemReader;
  use bitbuffer::BitBuffer;

  #[test]
  pub fn test_bit_buffer_read_bit(){
    let data = vec!(0b11001010, 0b11110000);
    println!("data: {}" ,data);
    let mut bb = BitBuffer::new(box MemReader::new(data));
    let bits = Vec::from_fn(16, |_|{
      bb.read_bit().unwrap()
    });
    assert_eq!(bits, vec!(1,1,0,0,1,0,1,0, 1,1,1,1,0,0,0,0));
  }
  #[test]
  pub fn test_bit_buffer_read_bits_lsb_byte(){
    let data = vec!(0b11001010, 0b11110000);
    let mut bb = BitBuffer::new(box MemReader::new(data));
    let bits = bb.read_bits_reversed(8).unwrap();
    assert_eq!(bits, 83);
  }
  #[test]
  pub fn test_bit_buffer_read_bits_lsb_less_then_byte(){
    let data = vec!(0b11001010, 0b11110000);
    let mut bb = BitBuffer::new(box MemReader::new(data));
    let bits = bb.read_bits_reversed(7).unwrap();
    assert_eq!(bits, 83);
  }
  #[test]
  pub fn test_bit_buffer_read_bits_msb_byte(){
    let data = vec!(0b11001010, 0b11110000);
    let mut bb = BitBuffer::new(box MemReader::new(data));
    let bits = bb.read_bits(8).unwrap();
    assert_eq!(bits, 202);
  }
  #[test]
  pub fn test_bit_buffer_read_bits_msb_less_then_byte(){
    let data = vec!(0b11001010, 0b11110000);
    let mut bb = BitBuffer::new(box MemReader::new(data));
    let bits = bb.read_bits(7).unwrap();
    assert_eq!(bits, 101);
  }
}