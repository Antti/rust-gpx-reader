use super::io_reader::IoReader;
use super::super::{Result, Error};
use super::song::{SongInfo, Song, Channel, MeasureHeader, TimeSignature, TripletFeel};

use std::default::Default;
use std::cmp::{max, min};

// A song consists of score information, triplet feel, tempo, song key,
// MIDI channels, measure and track count, measure headers, tracks,
// measures.
//
// -   Version: :ref:`byte-size-string` of size 30.
//
// -   Score information. See :meth:`readInfo`.
//
// -   Triplet feel: :ref:`bool`. If value is true, then triplet feel is
//     set to eigth.
//
// -   Tempo: :ref:`int`.
//
// -   Key: :ref:`int`. Key signature of the song.
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
    let song_info = try!(read_info(&mut io));
    let triplet_feel = match try!(io.read_bool()) {
        false => TripletFeel::None,
        true => TripletFeel::Eighth
    };
    let tempo = try!(io.read_int());
    let key = try!(io.read_int());
    let channels = try!(read_midi_channels(&mut io));
    let measure_count = try!(io.read_int());
    let track_count = try!(io.read_int());

    // try!(read_measure_headers(&mut io, measure_count));
    // try!(read_tracks(&mut io, track_count, channels);
    // try!(read_measures(&mut io, song));
    let song = Song {
        song_info: song_info, triplet_feel: Some(triplet_feel), tempo: tempo,
        channels: channels,
        measure_headers: vec![], tracks: vec![]
    };
    Ok(song)
}


// -   title
// -   subtitle
// -   artist
// -   album
// -   words
// -   copyright
// -   tabbed by
// -   instructions
pub fn read_info<T>(io: &mut T) -> Result<SongInfo> where T: IoReader {
    let title = try!(io.read_int_byte_sized_string());
    let subtitle = try!(io.read_int_byte_sized_string());
    let artist = try!(io.read_int_byte_sized_string());
    let album = try!(io.read_int_byte_sized_string());
    let words = try!(io.read_int_byte_sized_string());
    let copyright = try!(io.read_int_byte_sized_string());
    let tab = try!(io.read_int_byte_sized_string());
    let instructions = try!(io.read_int_byte_sized_string());
    let notes_count = try!(io.read_int());
    let mut notice = vec![];
    for _ in 0..notes_count {
        notice.push(try!(io.read_int_byte_sized_string()));
    }
    let song_info = SongInfo {
        title: title,
        subtitle: subtitle,
        artist: artist,
        album: album,
        words: words,
        music: None,
        copyright: copyright,
        tab: tab,
        instructions: instructions,
        notice: notice
    };
    Ok(song_info)
}


//
//
// Read MIDI channels.
//
//     Guitar Pro format provides 64 channels (4 MIDI ports by 16 channels),
//     the channels are stored in this order:
//
//     -   port1/channel1
//     -   port1/channel2
//     -   ...
//     -   port1/channel16
//     -   port2/channel1
//     -   ...
//     -   port4/channel16
//
//     Each channel has the following form:
//
//     -   Instrument: :ref:`int`.
//
//     -   Volume: :ref:`byte`.
//
//     -   Balance: :ref:`byte`.
//
//     -   Chorus: :ref:`byte`.
//
//     -   Reverb: :ref:`byte`.
//
//     -   Phaser: :ref:`byte`.
//
//     -   Tremolo: :ref:`byte`.
//
//     -   blank1: :ref:`byte`.
//
//     -   blank2: :ref:`byte`.
//

fn read_midi_channels<T>(io: &mut T) -> Result<Vec<Channel>> where T: IoReader {
    let mut channels = vec![];
    for i in 0..64 {
        let instrument = try!(io.read_int());
        // if newChannel.isPercussionChannel and instrument == -1:
        //     instrument = 0
        let volume = try!(io.read_signed_byte());
        let balance = try!(io.read_signed_byte());
        let chorus = try!(io.read_signed_byte());
        let reverb = try!(io.read_signed_byte());
        let phaser = try!(io.read_signed_byte());
        let tremolo = try!(io.read_signed_byte());
        let channel = Channel {
            channel: i,
            effect_channel: i,
            instrument: instrument,
            volume: volume,
            balance: balance,
            chorus: chorus,
            reverb: reverb,
            phaser: phaser,
            tremolo: tremolo
        };
        channels.push(channel);
        // Backward compatibility with version 3.0
        try!(io.skip(2));
    }
    Ok(channels)

}
//
// Read measure header.
//
// The first byte is the measure's flags. It lists the data given in the
// current measure.
//
// -   *0x01*: numerator of the key signature
// -   *0x02*: denominator of the key signature
// -   *0x04*: beginning of repeat
// -   *0x08*: end of repeat
// -   *0x10*: number of alternate ending
// -   *0x20*: presence of a marker
// -   *0x40*: tonality of the measure
// -   *0x80*: presence of a double bar
//
// Each of these elements is present only if the corresponding bit is a 1.
//
// The different elements are written (if they are present) from lowest to
// highest bit.
//
// Exceptions are made for the double bar and the beginning of repeat
// whose sole presence is enough, complementary data is not necessary.
//
// -   Numerator of the key signature: :ref:`byte`.
//
// -   Denominator of the key signature: :ref:`byte`.
//
// -   End of repeat: :ref:`byte`.
//     Number of repeats until the previous beginning of repeat.
//
// -   Number of alternate ending: :ref:`byte`.
//
// -   Marker: see :meth:`GP3File.readMarker`.
//
// -   Tonality of the measure: 2 :ref:`Bytes <byte>`. These values
//     encode a key signature change on the current piece. First byte is
//     key signature root, second is key signature type.


fn read_measure_headers<T>(io: &mut T, measure_count: u16) -> Result<Vec<MeasureHeader>> where T: IoReader {
    let mut previous : MeasureHeader = Default::default();
    for number in 1..measure_count + 1 {
        let flags = try!(io.read_byte());
        let start = 0;
        // header.tempo.value = song.tempo
        // header.tripletFeel = self._tripletFeel
        let numerator = if flags & 0x01 > 0 {
            try!(io.read_signed_byte())
        } else {
            previous.time_signature.numerator
        };
        let denominator = if flags & 0x02 > 0 {
            try!(io.read_signed_byte())
        } else {
            previous.time_signature.denominator
        };
        let time_signature = TimeSignature { numerator: numerator, denominator: denominator, ..Default::default()  };
        let is_repeat_open = (flags & 0x04 > 0);
        let repeat_close = if flags & 0x08 > 0 {
            try!(io.read_signed_byte()) > 0
        } else { false }; // TODO: Figure out if we need to use Option
        let repeat_alternative = if flags & 0x10 > 0 {
            //readRepeatAlternative
        };
        // TODO: Finish
    }
    Ok(vec![previous])
}

fn to_channel_short(data: u8) -> i16 {
    let value = max(-32768i16, min(32767i16, ((data as i16) << 3) - 1));
    max(value, -1) + 1
}
