extern crate std;
use std::io::{Reader, MemReader};
use bitbuffer;

pub enum GpxFileType {
  BCFS,
  BCFZ,
  Unknown
}

pub fn check_file_type(data: &[u8]) -> GpxFileType {
  match data.slice(0, 4) {
    [0x42, 0x43, 0x46, 0x53] => BCFS,
    [0x42, 0x43, 0x46, 0x5a] => BCFZ,
    _ => Unknown
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
    let to_read = std::cmp::min(len, offset);
    let slice = decomressed_data.slice(source_position, source_position+to_read).to_vec();
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
  std::io::stdio::stderr().write_line(format!("Successfully decompressed data. Len: {}, Expected len: {}", decomressed_data.len(), decomressed_data_len).as_slice()).unwrap();
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
    assert_eq!(::gpx::decompress_bcfz(data), vec!());
  }

  #[test]
  pub fn test_check_file_type(){
    use gpx::GpxFileType;
    let data_bcfs = [0x42, 0x43, 0x46, 0x53];
    let data_bcfz = [0x42, 0x43, 0x46, 0x5a];
    let data_random = [0xde, 0xad, 0xbe, 0xef];
    assert!(match ::gpx::check_file_type(data_bcfs) {
      ::gpx::BCFS => true,
      _ => false
    });
    assert!(match ::gpx::check_file_type(data_bcfz) {
      ::gpx::BCFZ => true,
      _ => false
    });
    assert!(match ::gpx::check_file_type(data_random) {
      ::gpx::Unknown => true,
      _ => false
    });
  }
}
