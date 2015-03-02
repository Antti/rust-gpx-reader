#![feature(old_io)]
#![feature(collections)]

#[macro_use]
extern crate log;
extern crate "rustc-serialize" as rustc_serialize;

use std::old_io::{IoResult, MemReader, SeekSet};
use std::cmp;
use std::str;

pub mod bitbuffer;

#[derive(Debug)]
pub enum GpxFileType {
  BCFS,
  BCFZ,
  Unknown
}

impl Copy for GpxFileType{}

#[derive(Debug, RustcEncodable)]
pub struct File {
  file_name: String,
  file_data: Vec<u8>
}

pub fn read(data: Vec<u8>) -> Result<Vec<File>, String> {
  debug!("Reading file...");
  match check_file_type(&data[..]){
    GpxFileType::BCFZ => {
      debug!("File type BCFZ");
      let data = data[4..].to_vec();
      let bcfs_data = match decompress_bcfz(data) {
        Err(err) => return Err(err.desc.to_string()),
        Ok(data) => data
      };
      match check_file_type(&bcfs_data[..]) {
        GpxFileType::BCFS => {
          debug!("Decompressed BCFZ, found BCFS inside");
          decompress_bcfs(bcfs_data[4..].to_vec()).map_err(|e| e.desc.to_string())
        },
        GpxFileType::BCFZ => Err("BCFZ in BCFZ, weird...".to_string()),
        GpxFileType::Unknown => Err("BCFZ file didn't contain BCFS inside".to_string())
      }
    },
    GpxFileType::BCFS => {
      debug!("File type BCFS");
      let data = data[4..].to_vec();
      decompress_bcfs(data).map_err(|e| e.desc.to_string())
    },
    GpxFileType::Unknown => Err("Unknown file type".to_string())
  }
}

pub fn check_file_type(data: &[u8]) -> GpxFileType {
  match &data[0..4] {
    [0x42, 0x43, 0x46, 0x53] => GpxFileType::BCFS,
    [0x42, 0x43, 0x46, 0x5a] => GpxFileType::BCFZ,
    _ => GpxFileType::Unknown
  }
}

pub fn decompress_bcfz(data: Vec<u8>) -> IoResult<Vec<u8>> {
  let mut bb = bitbuffer::BitBuffer::new(&data[..]);
  let expected_decomressed_data_len = try!(bb.read_le_i32()) as usize;
  let mut decomressed_data : Vec<u8> = Vec::with_capacity(expected_decomressed_data_len);
  debug!("Expected decomressed_data len: {}", expected_decomressed_data_len);

  #[inline]
  fn read_uncompressed_chunk<T: Reader>(bb: &mut bitbuffer::BitBuffer<T>, decomressed_data: &mut Vec<u8>) -> IoResult<()> {
    let len = try!(bb.read_bits_reversed(2));
    for _ in (0..len) {
      let byte = try!(bb.read_byte());
      decomressed_data.push(byte);
    };
    Ok(())
  }

  #[inline]
  fn read_compressed_chunk<T: Reader>(bb: &mut bitbuffer::BitBuffer<T>, decomressed_data: &mut Vec<u8>) -> IoResult<()> {
    let word_size = try!(bb.read_bits(4));
    let offset = try!(bb.read_bits_reversed(word_size));
    let len = try!(bb.read_bits_reversed(word_size));
    let source_position = decomressed_data.len() - offset;
    let to_read = cmp::min(len, offset);
    let slice = &decomressed_data[source_position..source_position+to_read].to_vec();
    decomressed_data.push_all(slice);
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
  debug!("Successfully decompressed data. Len: {}, Expected len: {}", decomressed_data.len(), expected_decomressed_data_len);
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
        file_data.extend(try!(reader.read_exact(sector_size as usize)).into_iter());
        block_count += 1;
      }

      try!(reader.seek(index_file_size, SeekSet));
      let file_size = try!(reader.read_le_i32()) as usize;
      if file_size <= file_data.len() {
        try!(reader.seek(index_file_name, SeekSet));
        let file_name = str::from_utf8(&try!(reader.read_exact(127))).unwrap().trim_right_matches('\0').to_string();
        try!(reader.seek(index_file_name, SeekSet));
        let file_bytes = &file_data[0..file_size];
        files.push(File{file_name: file_name.clone(), file_data: file_bytes.to_vec()});
      }
    }
  }
  Ok(files)
}

#[cfg(test)]
mod tests {
  #[allow(unreachable_code)]
  #[test]
  pub fn test_load_bcfz(){
    return;
    //NOT IMPLEMENTED. Need good source data example.
    let data = vec!();
    assert_eq!(super::decompress_bcfz(data).unwrap(), vec!());
  }

  #[test]
  pub fn test_check_file_type(){
    use super::GpxFileType;
    let data_bcfs = [0x42, 0x43, 0x46, 0x53];
    let data_bcfz = [0x42, 0x43, 0x46, 0x5a];
    let data_random = [0xde, 0xad, 0xbe, 0xef];
    assert!(match super::check_file_type(&data_bcfs) {
      GpxFileType::BCFS => true,
      _ => false
    });
    assert!(match super::check_file_type(&data_bcfz) {
      GpxFileType::BCFZ => true,
      _ => false
    });
    assert!(match super::check_file_type(&data_random) {
      GpxFileType::Unknown => true,
      _ => false
    });
  }
}
