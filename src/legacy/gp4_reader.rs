use super::io_reader::IoReader;
use super::super::{Result, Error};
use super::song::{SongInfo, Song, TripletFeel};
use super::gp3_reader;


// A song consists of score information, triplet feel, lyrics, tempo, song
// key, MIDI channels, measure and track count, measure headers,
// tracks, measures.
//
// -   Version: :ref:`byte-size-string` of size 30.
//
// -   Score information.
//     See :meth:`readInfo`.
//
// -   Triplet feel: :ref:`bool`.
//     If value is true, then triplet feel is set to eigth.
//
// -   Lyrics. See :meth:`readLyrics`.
//
// -   Tempo: :ref:`int`.
//
// -   Key: :ref:`int`. Key signature of the song.
//
// -   Octave: :ref:`signed-byte`. Reserved for future uses.
//
// -   MIDI channels. See :meth:`readMidiChannels`.
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
    let song_info = try!(gp3_reader::read_info(&mut io));
    let triplet_feel = if try!(io.read_bool()) {
        TripletFeel::Eighth
    } else {
        TripletFeel::None
    };
    let tempo = 0;
    let song = Song {
        song_info: song_info, triplet_feel: Some(triplet_feel),
        channels: vec![],
        tempo: tempo, measure_headers: vec![], tracks: vec![]
    };
    Ok(song)
}
