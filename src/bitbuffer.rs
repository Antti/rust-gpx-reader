use std::io::{self, Read, Cursor};

pub struct BitBuffer <'a> {
    bit_position: u8,
    byte: u8,
    cursor: Cursor<&'a [u8]>
}

impl <'a> Read for BitBuffer<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        for x in 0..buf.len(){
            buf[x] = try!(self.read_bits(8)) as u8;
        }
        Ok(buf.len())
    }
}

impl <'a> BitBuffer<'a> {
    pub fn new(data: &[u8]) -> BitBuffer {
        BitBuffer{ bit_position: 8, byte: 0, cursor: Cursor::new(data)}
    }

    // Reads bit one by one
    #[inline]
    pub fn read_bit(&mut self) -> io::Result<u8> {
        if self.bit_position == 8 {
            let buf = &mut [0u8];
            try!(self.cursor.read(buf));
            self.byte = buf[0];
            self.bit_position = 0;
        }
        let bit = (self.byte >> (7 - self.bit_position) as usize) & 0x1; //MSB
        self.bit_position += 1;
        Ok(bit)
    }

    // bigEndian MSB
    pub fn read_bits(&mut self, count: usize) -> io::Result<usize> {
        let mut word = 0usize;
        assert!(count <= 64);
        for idx in (0..count) {
            let bit = try!(self.read_bit());
            word = word | ((bit as usize) << (count - 1 - idx));
        }
        Ok(word)
    }

    pub fn read_bits_reversed(&mut self, count: usize) -> io::Result<usize> {
        let mut word = 0usize;
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
    use byteorder::{ReadBytesExt, BigEndian, LittleEndian};

    #[test]
    pub fn test_bit_buffer_read_bit(){
        let data : &[u8] = &[0b11001010, 0b11110000];
        println!("data: {:?}", data);
        let mut bb = BitBuffer::new(data);
        let bits : Vec<u8> = (0..16).map(|_|{
            bb.read_bit().unwrap()
        }).collect();
        assert_eq!(bits, vec!(1,1,0,0,1,0,1,0, 1,1,1,1,0,0,0,0));
    }
    #[test]
    pub fn test_bit_buffer_read_bits_lsb_byte(){
        let data : &[u8] = &[0b11001010, 0b11110000];
        let mut bb = BitBuffer::new(data);
        let bits = bb.read_bits_reversed(8).unwrap();
        assert_eq!(bits, 83);
    }
    #[test]
    pub fn test_bit_buffer_read_bits_lsb_less_then_byte(){
        let data : &[u8] = &[0b11001010, 0b11110000];
        let mut bb = BitBuffer::new(data);
        let bits = bb.read_bits_reversed(7).unwrap();
        assert_eq!(bits, 83);
    }
    #[test]
    pub fn test_bit_buffer_read_bits_msb_byte(){
        let data : &[u8] = &[0b11001010, 0b11110000];
        let mut bb = BitBuffer::new(data);
        let bits = bb.read_bits(8).unwrap();
        assert_eq!(bits, 202);
    }
    #[test]
    pub fn test_bit_buffer_read_bits_msb_less_then_byte(){
        let data : &[u8] = &[0b11001010, 0b11110000];
        let mut bb = BitBuffer::new(data);
        let bits = bb.read_bits(7).unwrap();
        assert_eq!(bits, 101);
    }
    #[test]
    pub fn test_bit_buffer_read_u16_lsb(){
        let data : &[u8] = &[0b11001010, 0b11110000, 0b11110000];
        let mut bb = BitBuffer::new(data);
        bb.read_bits(2).unwrap();
        let num = bb.read_u16::<LittleEndian>().unwrap();  //00101011_11000011
        assert_eq!(num, 49963);
    }
    #[test]
    pub fn test_bit_buffer_read_u16_msb(){
        let data : &[u8] = &[0b11001010, 0b11110000, 0b11110000];
        let mut bb = BitBuffer::new(data);
        bb.read_bits(2).unwrap();
        let num = bb.read_u16::<BigEndian>().unwrap();  //00101011_11000011
        assert_eq!(num, 11203);
    }
}
