use super::io_reader::IoReader;
use super::super::{Result, Error, ErrorKind};
use super::Song;
use super::version::Version;
use super::{gp3_reader, gp4_reader, gp5_reader};

pub struct GPFile <T> where T: IoReader {
    io: T
}

impl <T> GPFile<T> where T: IoReader {
    pub fn new(data: T) -> Self {
        GPFile { io: data }
    }

    pub fn read(mut self) -> Result<(Version, Song)> {
        let version = try!(self.read_version());
        let song = match version {
            Version::FichierGuitarProV300 => gp3_reader::read(self.io),
            Version::FichierGuitarProV400 | Version::FichierGuitarProV406 | Version::FichierGuitarProL406 => gp4_reader::read(self.io),
            Version::FichierGuitarProV500 | Version::FichierGuitarProV510 => gp5_reader::read(self.io)
        };
        Ok((version, try!(song)))
    }

    fn read_version(&mut self) -> Result<Version> {
        match &try!(self.io.read_byte_sized_string(30)) as &str {
            "FICHIER GUITAR PRO v3.00" => Ok(Version::FichierGuitarProV300),
            "FICHIER GUITAR PRO v4.00" => Ok(Version::FichierGuitarProV400),
            "FICHIER GUITAR PRO v4.06" => Ok(Version::FichierGuitarProV406),
            "FICHIER GUITAR PRO L4.06" => Ok(Version::FichierGuitarProL406),
            "FICHIER GUITAR PRO v5.00" => Ok(Version::FichierGuitarProV500),
            "FICHIER GUITAR PRO v5.10" => Ok(Version::FichierGuitarProV510),
            unknown => Err(ErrorKind::FormatError(format!("Unsupported version: {}", unknown)).into())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{GPFile};
    use super::super::version::Version;
    use std::fs::File;
    use std::path::Path;

    #[test]
    fn test_read_version(){
        let file = File::open(&Path::new("test_data/Iron Maiden - Fear Of The Dark (Pro).gp4")).unwrap();
        let mut gp_file = GPFile::new(file);
        let (version, song) = gp_file.read().unwrap();
        assert_eq!(version, Version::FichierGuitarProV406);
    }
}
