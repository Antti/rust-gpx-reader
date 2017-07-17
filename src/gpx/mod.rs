use std::io::{Cursor, Read};
use std::cmp;
use std::iter;

use byteorder::{ReadBytesExt, LittleEndian};
use super::bitbuffer;
use super::{Result, ErrorKind};

#[derive(Debug, Clone, Copy)]
pub enum GpxFileType {
    BCFS,
    BCFZ,
}

#[derive(Debug)]
pub struct File {
    file_name: String,
    file_data: Vec<u8>,
}

pub fn read(data: &[u8]) -> Result<Vec<File>> {
    debug!("Reading file...");
    match check_file_type(data) {
        Some(GpxFileType::BCFZ) => {
            debug!("File type BCFZ");
            let bcfs_data = decompress_bcfz(&data[4..])?;
            match check_file_type(&bcfs_data) {
                Some(GpxFileType::BCFS) => {
                    debug!("Decompressed BCFZ, found BCFS inside");
                    decompress_bcfs(&bcfs_data[4..])
                }
                Some(GpxFileType::BCFZ) => Err(ErrorKind::FormatError("BCFZ in BCFZ, weird...".to_string()).into()),
                None => Err(ErrorKind::FormatError("BCFZ file didn't contain BCFS inside".to_string()).into()),
            }
        }
        Some(GpxFileType::BCFS) => {
            debug!("File type BCFS");
            decompress_bcfs(&data[4..])
        }
        None => Err(ErrorKind::FormatError("Uknown file format".to_string()).into()),
    }
}

pub fn check_file_type(data: &[u8]) -> Option<GpxFileType> {
    match (data[0], data[1], data[2], data[3]) {
        (0x42, 0x43, 0x46, 0x53) => Some(GpxFileType::BCFS),
        (0x42, 0x43, 0x46, 0x5a) => Some(GpxFileType::BCFZ),
        _ => None,
    }
}

pub fn decompress_bcfz(data: &[u8]) -> Result<Vec<u8>> {
    let mut bb = bitbuffer::BitBuffer::new(data);
    let expected_decompressed_data_len = bb.read_i32::<LittleEndian>()? as usize;
    let mut decompressed_data: Vec<u8> = Vec::with_capacity(expected_decompressed_data_len);
    debug!("Expected decompressed_data len: {}",
           expected_decompressed_data_len);

    #[inline]
    fn read_uncompressed_chunk(bb: &mut bitbuffer::BitBuffer, decompressed_data: &mut Vec<u8>) -> Result<()> {
        let len = bb.read_bits_reversed(2)?;
        let mut buf: Vec<_> = iter::repeat(0u8).take(len).collect();
        bb.read_exact(&mut buf)?;
        decompressed_data.extend(buf);
        Ok(())
    }

    #[inline]
    fn read_compressed_chunk(bb: &mut bitbuffer::BitBuffer, decompressed_data: &mut Vec<u8>) -> Result<()> {
        let word_size = bb.read_bits(4)?;
        let offset = bb.read_bits_reversed(word_size)?;
        let len = bb.read_bits_reversed(word_size)?;
        assert!(decompressed_data.len() >= offset);
        let source_position = decompressed_data.len() - offset;
        let to_read = cmp::min(len, offset);
        let slice = &decompressed_data[source_position..source_position + to_read].to_vec();
        decompressed_data.extend(slice);
        Ok(())
    }

    while decompressed_data.len() < expected_decompressed_data_len {
        let bit = bb.read_bit()?;
        match bit {
            0 => read_uncompressed_chunk(&mut bb, &mut decompressed_data)?,
            1 => read_compressed_chunk(&mut bb, &mut decompressed_data)?,
            _ => unreachable!(),
        }
    }
    debug!("Successfully decompressed data. Len: {}, Expected len: {}",
           decompressed_data.len(),
           expected_decompressed_data_len);
    Ok(decompressed_data)
}

pub fn decompress_bcfs(data: &[u8]) -> Result<Vec<File>> {
    let data_len = data.len() as u64;
    let sector_size = 0x1000u64;
    let mut reader = Cursor::new(data);
    let mut offset = 0u64;
    let mut files = vec![];

    loop {
        offset += sector_size;
        if offset + 3 >= data_len {
            break;
        }
        reader.set_position(offset);
        if reader.read_i32::<LittleEndian>()? == 2 {
            let index_file_name = offset + 4;
            let index_file_size = offset + 0x8C;
            let index_of_block = offset + 0x94;
            let mut file_data: Vec<u8> = Vec::new();

            let mut block;
            let mut block_count = 0u64;
            loop {
                reader.set_position(index_of_block + (4 * block_count));
                block = reader.read_i32::<LittleEndian>()?;
                if block == 0 {
                    break;
                }
                offset = (block as u64) * sector_size;
                reader.set_position(offset);
                let mut buf = vec![0; sector_size as usize];
                reader.read_exact(&mut buf)?;
                file_data.extend(buf);
                block_count += 1;
            }

            reader.set_position(index_file_size);
            let file_size = reader.read_i32::<LittleEndian>()? as usize;
            if file_size <= file_data.len() {
                reader.set_position(index_file_name);
                let mut buf = vec![0; 127];
                reader.read_exact(&mut buf)?;
                let file_name = String::from_utf8_lossy(&buf).trim_right_matches('\0').to_owned();
                reader.set_position(index_file_name);
                let file_bytes = &file_data[0..file_size];
                files.push(File {
                               file_name: file_name.clone(),
                               file_data: file_bytes.to_vec(),
                           });
            }
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    #[allow(unreachable_code)]
    #[test]
    pub fn test_load_bcfz() {
        //NOT IMPLEMENTED. Need good source data example.
        // let data = vec!();
        // assert_eq!(super::decompress_bcfz(&data).unwrap(), vec!());
    }

    #[test]
    pub fn test_check_file_type() {
        use super::GpxFileType;
        let data_bcfs = [0x42, 0x43, 0x46, 0x53];
        let data_bcfz = [0x42, 0x43, 0x46, 0x5a];
        let data_random = [0xde, 0xad, 0xbe, 0xef];
        assert!(match super::check_file_type(&data_bcfs) {
                    Some(GpxFileType::BCFS) => true,
                    _ => false,
                });
        assert!(match super::check_file_type(&data_bcfz) {
                    Some(GpxFileType::BCFZ) => true,
                    _ => false,
                });
        assert!(match super::check_file_type(&data_random) {
                    None => true,
                    _ => false,
                });
    }
}
