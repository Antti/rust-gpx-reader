extern crate serialize;

use std::io::{IoResult,MemReader, SeekSet};
use std::cmp;
use std::str;
use bitbuffer;

pub enum GpxFileType {
  BCFS,
  BCFZ,
  Unknown
}

#[deriving(Show, Encodable)]
pub struct File {
  file_name: String,
  file_data: Vec<u8>
}

pub fn read(data: Vec<u8>) -> Result<Vec<File>, String> {
  match check_file_type(data.as_slice()){
    BCFZ => {
      let data = Vec::from_slice(data.slice_from(4));
      let bcfs_data = match decompress_bcfz(data) {
        Err(err) => return Err(err.desc.to_string()),
        Ok(data) => data
      };
      match check_file_type(bcfs_data.as_slice()) {
        BCFS => {
          decompress_bcfs(bcfs_data.slice_from(4).to_vec()).map_err(|e| e.desc.to_string())
        },
        BCFZ => Err("BCFZ in BCFZ, weird...".to_string()),
        Unknown => Err("BCFZ file didn't contain BCFS inside".to_string())
      }
    },
    BCFS => {
      let data = Vec::from_slice(data.slice_from(4));
      decompress_bcfs(data).map_err(|e| e.desc.to_string())
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

pub fn decompress_bcfz(data: Vec<u8>) -> IoResult<Vec<u8>> {
  let mut bb = bitbuffer::BitBuffer::new(box MemReader::new(data));
  let expected_decomressed_data_len = try!(bb.read_le_i32()) as uint;
  let mut decomressed_data : Vec<u8> = Vec::with_capacity(expected_decomressed_data_len);
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
    let to_read = cmp::min(len, offset);
    let slice = decomressed_data.slice(source_position, source_position+to_read).to_vec();
    decomressed_data.push_all(slice.as_slice());
    Ok(())
  }

  while decomressed_data.len() < expected_decomressed_data_len {
    let bit = try!(bb.read_bit());
    match bit {
      0 => { try!(read_uncompressed_chunk(&mut bb, &mut decomressed_data)) },
      1 => { try!(read_compressed_chunk(&mut bb, &mut decomressed_data)) },
      _ => unreachable!()
    }
  }
  // stdio::stderr().write_line(format!("Successfully decompressed data. Len: {}, Expected len: {}", decomressed_data.len(), expected_decomressed_data_len).as_slice()).unwrap();
  Ok(decomressed_data)
}

pub fn decompress_bcfs(data: Vec<u8>) -> IoResult<Vec<File>> {
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
    try!(reader.seek(offset, SeekSet));
    if try!(reader.read_le_i32()) == 2 {
      let index_file_name = offset + 4;
      let index_file_size = offset + 0x8C;
      let index_of_block = offset + 0x94;
      let mut file_data : Vec<u8> = Vec::new();

      let mut block;
      let mut block_count = 0i64;
      loop {
        try!(reader.seek(index_of_block + (4*block_count), SeekSet));
        block = try!(reader.read_le_i32());
        if block == 0 {
          break;
        }
        offset = (block as i64) * sector_size;
        try!(reader.seek(offset, SeekSet));
        file_data = file_data.append(try!(reader.read_exact(sector_size as uint)).as_slice());
        block_count += 1;
      }

      try!(reader.seek(index_file_size, SeekSet));
      let file_size = try!(reader.read_le_i32()) as uint;
      if file_size <= file_data.len() {
        try!(reader.seek(index_file_name, SeekSet));
        let file_name = str::from_utf8(try!(reader.read_exact(127)).as_slice()).unwrap().trim_right_chars('\0').to_string();
        try!(reader.seek(index_file_name, SeekSet));
        let file_bytes = file_data.slice(0, file_size);
        files.push(File{file_name: file_name.clone(), file_data: file_bytes.to_vec()});
      }
    }
  }
  Ok(files)
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
    assert_eq!(::gpx::decompress_bcfz(data).unwrap(), vec!());
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
