use super::io_reader::IoReader;
use super::super::{Result, Error};
use super::song::{SongInfo, Song};
use super::version::Version;

//
// A song consists of score information, triplet feel, lyrics, tempo, song
// key, MIDI channels, measure and track count, measure headers,
// tracks, measures.
//
// -   Version: :ref:`byte-size-string` of size 30.
//
// -   Score information.
//     See :meth:`readInfo`.
//
// -   Lyrics. See :meth:`readLyrics`.
//
// -   RSE master effect. See :meth:`readRSEInstrument`.
//
// -   Tempo name: :ref:`int-byte-size-string`.
//
// -   Tempo: :ref:`int`.
//
// -   Hide tempo: :ref:`bool`. Don't display tempo on the sheet if set.
//
// -   Key: :ref:`int`. Key signature of the song.
//
// -   Octave: :ref:`int`. Octave of the song.
//
// -   MIDI channels. See :meth:`readMidiChannels`.
//
// -   Directions. See :meth:`readDirections`.
//
// -   Master reverb. See :meth:`readMasterReverb`.
//
// -   Number of measures: :ref:`int`.
//
// -   Number of tracks: :ref:`int`.
//
// -   Measure headers. See :meth:`readMeasureHeaders`.
//
// -   Tracks. See :meth:`readTracks`.
//
// -   Measures. See :meth:`readMeasures`.



pub fn read<T>(mut io: T) -> Result<Song> where T: IoReader {
    let song_info = try!(read_info(&mut io));
    let tempo = 0;
    let song = Song {
        song_info: song_info, triplet_feel: None,
        channels: vec![],
        tempo: tempo, measure_headers: vec![], tracks: vec![]
    };
    Ok(song)
}

//
// -   title
// -   subtitle
// -   artist
// -   album
// -   words
// -   music
// -   copyright
// -   tabbed by
// -   instructions
fn read_info<T>(io: &mut T) -> Result<SongInfo> where T: IoReader {
    let title = try!(io.read_int_byte_sized_string());
    let subtitle = try!(io.read_int_byte_sized_string());
    let artist = try!(io.read_int_byte_sized_string());
    let album = try!(io.read_int_byte_sized_string());
    let words = try!(io.read_int_byte_sized_string());
    let music = try!(io.read_int_byte_sized_string());
    let copyright = try!(io.read_int_byte_sized_string());
    let tab = try!(io.read_int_byte_sized_string());
    let instructions = try!(io.read_int_byte_sized_string());
    let song_info = SongInfo {
        title: title,
        subtitle: subtitle,
        artist: artist,
        album: album,
        words: words,
        music: Some(music),
        copyright: copyright,
        tab: tab,
        instructions: instructions,
        notice: vec![]
    };
    Ok(song_info)
}
