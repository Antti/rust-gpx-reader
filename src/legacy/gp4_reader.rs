use super::io_reader::IoReader;
use super::super::Result;
use super::song::{Song, TripletFeel, Lyrics, LyricsItem};
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
// -   Triplet feel:    :ref:`bool`.
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

pub use self::gp3_reader::read_info;

//       _______________________________________________________
//      |        |                                               |
//      |        | Version                                       |
//      |        |_______________________________________________|
//      |        |                                               |
//      |        | Tablature                                     |
//      |        |_______________________________________________|
//      |Headers |                                               |
//      |        | Lyrics                                        |
//      |        |_______________________________________________|
//      |        |                                               |
//      |        | Other Tablature Information                   |
//      |________|_______________________________________________|
// File |        |                                               |
//      |        | Measures                                      |
//      |        |_______________________________________________|
//      |        |                                               |
//      |        | Tracks                                        |
//      |        |_______________________________________________|
//      |        |               |               |               |
//      |        |               |               | Note 1        |
//      |Body    |               | Beat 1        |_______________|
//      |        |               |               |               |
//      |        |               |               | Note i ...    |
//      |        | Measure-Track |_______________|_______________|
//      |        | Pairs         |               |               |
//      |        |               |               | Note 1        |
//      |        |               | Beat i ...    |_______________|
//      |        |               |               |               |
//      |        |               |               | Note i ...    |
//      |________|_______________|_______________|_______________|
//      |                                                        |
//      | Chord Diagrams                                         |
//      |________________________________________________________|

pub fn read<T>(mut io: T) -> Result<Song>
    where T: IoReader
{
    // Headers
    let song_info = read_info(&mut io)?;
    // Triplet feel
    let triplet_feel = if io.read_bool()? {
        TripletFeel::Eighth
    } else {
        TripletFeel::None
    };
    // Lyrics
    let lyrics_track = io.read_int()?;
    let mut lyrics = Lyrics { track: lyrics_track as u32, lyrics: vec![] };
    for _ in 0..5 {
        let starting_measure = io.read_int()?;
        let text = io.read_int_sized_string()?;
        let lyrics_item = LyricsItem {
            starting_measure: starting_measure as u32,
            text
        };
        lyrics.lyrics.push(lyrics_item);
    }
    println!("{:?}", lyrics);
    let tempo = io.read_int()?;
    // song.key = gp.KeySignature((self.readInt(), 0))
    // self.readSignedByte()  # octave
    // channels = self.readMidiChannels()
    // measureCount = self.readInt()
    // trackCount = self.readInt()
    // self.readMeasureHeaders(song, measureCount)
    // self.readTracks(song, trackCount, channels)
    // self.readMeasures(song)
    let song = Song {
        song_info: song_info,
        triplet_feel: Some(triplet_feel),
        channels: vec![],
        tempo: tempo,
        measure_headers: vec![],
        tracks: vec![],
    };
    Ok(song)
}
