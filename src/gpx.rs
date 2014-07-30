extern crate std;
use std::io::{IoResult,MemReader};
use bitbuffer;

pub enum GpxFileType {
  BCFS,
  BCFZ,
  Unknown
}

#[deriving(Show)]
#[deriving(Decodable, Encodable)]
pub struct File {
  file_name: String,
  file_data: Vec<u8>
}

pub fn read(data: Vec<u8>) -> Result<Vec<File>, String> {
  match check_file_type(data.as_slice()){
    BCFZ => {
      let data = Vec::from_slice(data.tailn(4));
      let content = Vec::from_slice(decompress_bcfz(data).tailn(4));
      Ok(decompress_bcfs(content))
    },
    BCFS => {
      let data = Vec::from_slice(data.tailn(4));
      Ok(decompress_bcfs(data))
    },
    Unknown => Err("Unknown file type".to_string())
  }
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

  #[inline]
  fn read_uncompressed_chunk(bb: &mut bitbuffer::BitBuffer, decomressed_data: &mut Vec<u8>) -> IoResult<()> {
    let len = try!(bb.read_bits_reversed(2));
    for _ in range(0,len) {
      let byte = try!(bb.read_byte());
      decomressed_data.push(byte);
    };
    Ok(())
  }

  #[inline]
  fn read_compressed_chunk(bb: &mut bitbuffer::BitBuffer, decomressed_data: &mut Vec<u8>) -> IoResult<()> {
    let word_size = try!(bb.read_bits(4));
    let offset = try!(bb.read_bits_reversed(word_size));
    let len = try!(bb.read_bits_reversed(word_size));
    let source_position = decomressed_data.len() - offset;
    let to_read = std::cmp::min(len, offset);
    let slice = decomressed_data.slice(source_position, source_position+to_read).to_vec();
    decomressed_data.push_all(slice.as_slice());
    Ok(())
  }

  while decomressed_data.len() < decomressed_data_len {
    match bb.read_bit() {
      Ok(x) => match x {
        0 => { read_uncompressed_chunk(&mut bb, &mut decomressed_data).is_ok() || return decomressed_data; },
        1 => { read_compressed_chunk(&mut bb, &mut decomressed_data).is_ok() || return decomressed_data; },
        _ => unreachable!()
      },
      Err(_) => return decomressed_data
    }
  }
  // std::io::stdio::stderr().write_line(format!("Successfully decompressed data. Len: {}, Expected len: {}", decomressed_data.len(), decomressed_data_len).as_slice()).unwrap();
  decomressed_data
}

pub fn decompress_bcfs(data: Vec<u8>) -> Vec<File> {
  let data_len = data.len() as i64;
  let sector_size = 0x1000i64;
  let mut reader = MemReader::new(data);
  let mut offset = 0i64;
  let mut files : Vec<File> = vec!();

  loop {
    offset = offset + sector_size;
    if offset + 3 >= data_len {
      break;
    }
    reader.seek(offset, std::io::SeekSet).unwrap();
    if reader.read_le_i32().unwrap() == 2 {
      let index_file_name = offset + 4;
      let index_file_size = offset + 0x8C;
      let index_of_block = offset + 0x94;
      let mut file_data : Vec<u8> = Vec::new();

      let mut block;
      let mut block_count = 0i64;
      loop {
        reader.seek(index_of_block + (4*block_count), std::io::SeekSet).unwrap();
        block = reader.read_le_i32().unwrap();
        if block == 0 {
          break;
        }
        offset = (block as i64) * sector_size;
        reader.seek(offset, std::io::SeekSet).unwrap();
        file_data = file_data.append(reader.read_exact(sector_size as uint).unwrap().as_slice());
        block_count += 1;
      }

      reader.seek(index_file_size, std::io::SeekSet).unwrap();
      let file_size = reader.read_le_i32().unwrap() as uint;
      if file_size <= file_data.len() {
        reader.seek(index_file_name, std::io::SeekSet).unwrap();
        let file_name = std::str::from_utf8(reader.read_exact(127).unwrap().as_slice()).unwrap().trim_right_chars('\0').to_string();
        reader.seek(index_file_name, std::io::SeekSet).unwrap();
        let file_bytes = file_data.slice(0, file_size);
        files.push(File{file_name: file_name.clone(), file_data: file_bytes.to_vec()});
      }
    }
  }
  files
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
