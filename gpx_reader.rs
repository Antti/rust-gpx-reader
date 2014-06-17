extern crate core;
use std::vec::Vec;
use std::io::{Reader, MemReader};

pub mod bitbuffer {
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
      let bit = (self.byte >> (8 - self.bit_position - 1)) & 0x1; //MSB
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
}

pub fn decompress_bcfz(data: Vec<u8>) -> Vec<u8> {
  let mut bb = bitbuffer::BitBuffer::new(box MemReader::new(data));
  let decomressed_data_len = bb.read_le_i32().unwrap();
  let mut decomressed_data : Vec<u8> = Vec::with_capacity(decomressed_data_len);
  // println!("Expected decomressed_data len: {}", decomressed_data_len);

  fn read_uncompressed_chunk(bb: &mut bitbuffer::BitBuffer, decomressed_data: &mut Vec<u8>) -> bool {
    let len = match bb.read_bits_reversed(2){
      Some(x) => x,
      None => return false
    };
    for _ in range(0,len) {
      let byte = match bb.read_byte(){
        Some(x) => x,
        None => return false
      };
      decomressed_data.push(byte);
    }
    true
  }

  fn read_compressed_chunk(bb: &mut bitbuffer::BitBuffer, decomressed_data: &mut Vec<u8>) -> bool {
    let word_size = match bb.read_bits(4){
      Some(x) => x,
      None => return false
    };
    let offset = match bb.read_bits_reversed(word_size){
      Some(x) => x,
      None => return false
    };
    let len = match bb.read_bits_reversed(word_size){
      Some(x) => x,
      None => return false
    };
    let source_position = decomressed_data.len() - offset;
    let to_read = core::cmp::min(len, offset);
    let slice = decomressed_data.slice(source_position, source_position+to_read).to_owned();
    decomressed_data.push_all(slice.as_slice());
    true
  }

  while decomressed_data.len() < decomressed_data_len {
    match bb.read_bit() {
      Some(x) => match x {
        0 => { read_uncompressed_chunk(&mut bb, &mut decomressed_data) || return decomressed_data; },
        1 => { read_compressed_chunk(&mut bb, &mut decomressed_data) || return decomressed_data; },
        _ => unreachable!()
      },
      None => return decomressed_data
    }
  }
  println!("Successfully decompressed data. Len: {}, Expected len: {}", decomressed_data.len(), decomressed_data_len);
  decomressed_data
}


#[cfg(test)]
mod tests {
  use std::io::MemReader;
  use bitbuffer::BitBuffer;
  #[allow(unreachable_code)]
  #[test]
  pub fn test_load_bcfz(){
    return;
    //NOT IMPLEMENTED. Need good source data example.
    let data = vec!();
    assert_eq!(::decompress_bcfz(data), vec!());
  }

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

#[allow(unused_must_use)]
#[cfg(not(test))]
fn main(){
  use std::io::fs::File;
  let stream = if std::os::args().len() > 1{
    File::open(&Path::new(std::os::args().get(1).as_slice())).read_to_end()
  }else{
    let mut stdin = std::io::stdio::stdin();
    stdin.read_to_end()
  };
  let data = Vec::from_slice(stream.unwrap().tailn(4));
  let content = decompress_bcfz(data);
  let mut stdout = std::io::stdio::stdout();
  stdout.write(content.as_slice());
}
