use super::io_reader::IoReader;
use super::super::Result;

pub struct GPFile <T> where T: IoReader {
    io: T
}

impl <T> GPFile<T> where T: IoReader {
    pub fn new(data: T) -> Self {
        GPFile { io: data }
    }

    pub fn read_version(&mut self) -> Result<String> {
        self.io.read_byte_sized_string(30)
    }
}


#[cfg(test)]
mod tests {
    use super::GPFile;
    use std::fs::File;
    use std::path::Path;

    #[test]
    fn test_read_version(){
        let file = File::open(&Path::new("test_data/Iron Maiden - Fear Of The Dark (Pro).gp4")).unwrap();
        let mut gp_file = GPFile::new(file);
        assert_eq!(&gp_file.read_version().unwrap() as &str, "FICHIER GUITAR PRO v4.06");
    }
}
