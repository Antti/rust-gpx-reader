use super::io_reader::IoReader;
use super::super::{Result, Error};
use super::song::SongInfo;

#[derive(PartialEq, Debug)]
pub enum Version {
    FichierGuitarProV300,
    FichierGuitarProV400,
    FichierGuitarProV406,
    FichierGuitarProL406,
    FichierGuitarProV500,
    FichierGuitarProV510
}

pub struct GPFile <T> where T: IoReader {
    io: T,
    pub version: Version
}

impl <T> GPFile<T> where T: IoReader {
    pub fn new(data: T) -> Self {
        GPFile { io: data, version: Version::FichierGuitarProV300 }
    }

    pub fn read(&mut self) -> Result<SongInfo> {
        self.version = try!(self.read_version());
        self.read_info()
    }

    fn read_version(&mut self) -> Result<Version> {
        match &try!(self.io.read_byte_sized_string(30)) as &str {
            "FICHIER GUITAR PRO v3.00" => Ok(Version::FichierGuitarProV300),
            "FICHIER GUITAR PRO v4.00" => Ok(Version::FichierGuitarProV400),
            "FICHIER GUITAR PRO v4.06" => Ok(Version::FichierGuitarProV406),
            "FICHIER GUITAR PRO L4.06" => Ok(Version::FichierGuitarProL406),
            "FICHIER GUITAR PRO v5.00" => Ok(Version::FichierGuitarProV500),
            "FICHIER GUITAR PRO v5.10" => Ok(Version::FichierGuitarProV510),
            unknown => Err(Error::FormatError(format!("Unsupported version: {}", unknown)))
        }
    }

    // Read score information.
    //
    // Score information consists of sequence of IntByteSizeStrings
    //  <int-byte-size-string>:
    //  -   title
    //  -   subtitle
    //  -   artist
    //  -   album
    //  -   words
    //  -   music  (only gp5)
    //  -   copyright
    //  -   tabbed by
    //  -   instructions
    //  The sequence if followed by notice. Notice starts with the number of
    //  notice lines stored in int. Each line is encoded in
    //  nt-byte-size-string.
    fn read_info(&mut self) -> Result<SongInfo> {
        let title = try!(self.io.read_int_byte_sized_string());
        let subtitle = try!(self.io.read_int_byte_sized_string());
        let artist = try!(self.io.read_int_byte_sized_string());
        let album = try!(self.io.read_int_byte_sized_string());
        let words = try!(self.io.read_int_byte_sized_string());
        let music = match self.version {
            Version::FichierGuitarProV500 | Version::FichierGuitarProV510 => Some(try!(self.io.read_int_byte_sized_string())),
            _ => None
        };
        let copyright = try!(self.io.read_int_byte_sized_string());
        let tab = try!(self.io.read_int_byte_sized_string());
        let instructions = try!(self.io.read_int_byte_sized_string());
        let song_info = SongInfo {
            title: title,
            subtitle: subtitle,
            artist: artist,
            album: album,
            words: words,
            music: music,
            copyright: copyright,
            tab: tab,
            instructions: instructions,
            notice: vec![]
        };
        Ok(song_info)
    }
}


#[cfg(test)]
mod tests {
    use super::{GPFile, Version};
    use std::fs::File;
    use std::path::Path;

    #[test]
    fn test_read_version(){
        let file = File::open(&Path::new("test_data/Iron Maiden - Fear Of The Dark (Pro).gp4")).unwrap();
        let mut gp_file = GPFile::new(file);
        gp_file.read();
        assert_eq!(gp_file.version, Version::FichierGuitarProV406);
    }
}
