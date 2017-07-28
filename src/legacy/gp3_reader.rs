use super::io_reader::IoReader;
use super::super::Result;
use super::song::{SongInfo, Song, Channel, MeasureHeader, TimeSignature, TripletFeel, Duration, KeySignature, Marker, Color, Track, GuitarString};

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

pub fn read<T>(mut io: T) -> Result<Song>
    where T: IoReader
{
    let song_info = read_info(&mut io)?;
    let triplet_feel = if io.read_bool()? {
        TripletFeel::Eighth
    } else {
        TripletFeel::None
    };
    let tempo = io.read_int()?;
    let key = io.read_int()?;
    let mut channels = read_midi_channels(&mut io)?;
    let measure_count = io.read_int()?;
    let track_count = io.read_int()?;

    let measure_headers = read_measure_headers(&mut io, measure_count as u16, tempo as u16, triplet_feel)?;
    let tracks = read_tracks(&mut io, track_count, &mut channels)?;
    // let measures = read_measures(&mut io)?;
    let song = Song {
        song_info: song_info,
        triplet_feel: Some(triplet_feel),
        tempo: tempo,
        channels: channels,
        measure_headers: measure_headers,
        tracks: tracks,
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
pub fn read_info<T>(io: &mut T) -> Result<SongInfo>
    where T: IoReader
{
    let title = io.read_int_byte_sized_string()?;
    let subtitle = io.read_int_byte_sized_string()?;
    let artist = io.read_int_byte_sized_string()?;
    let album = io.read_int_byte_sized_string()?;
    let words = io.read_int_byte_sized_string()?;
    let copyright = io.read_int_byte_sized_string()?;
    let tab = io.read_int_byte_sized_string()?;
    let instructions = io.read_int_byte_sized_string()?;
    let notice_count = io.read_int()?;
    let mut notice = vec![];
    for _ in 0..notice_count {
        notice.push(io.read_int_byte_sized_string()?);
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
        notice: notice,
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

fn read_midi_channels<T>(io: &mut T) -> Result<Vec<Channel>>
    where T: IoReader
{
    let mut channels = vec![];
    for i in 0..64 {
        let instrument = io.read_int()?;
        // if newChannel.isPercussionChannel and instrument == -1:
        //     instrument = 0
        let volume = io.read_signed_byte()?;
        let balance = io.read_signed_byte()?;
        let chorus = io.read_signed_byte()?;
        let reverb = io.read_signed_byte()?;
        let phaser = io.read_signed_byte()?;
        let tremolo = io.read_signed_byte()?;
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
        io.skip(2)?;
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


fn read_measure_headers<T>(io: &mut T, measure_count: u16, song_tempo: u16, song_triplet_feel: TripletFeel) -> Result<Vec<MeasureHeader>>
    where T: IoReader
{
    let mut measure_headers = vec![];
    let mut previous: MeasureHeader = Default::default();
    for number in 1..measure_count + 1 {
        let flags = io.read_byte()?;
        let numerator = if flags & 0x01 > 0 {
            io.read_signed_byte()?
        } else {
            previous.time_signature.numerator
        };
        let denominator = if flags & 0x02 > 0 {
            io.read_signed_byte()?
        } else {
            previous.time_signature.denominator
        };
        let time_signature = TimeSignature {
            numerator: numerator,
            denominator: denominator,
            ..Default::default()
        };
        let is_repeat_open = flags & 0x04 > 0;
        let repeat_close = if flags & 0x08 > 0 {
            io.read_signed_byte()? > 0
        } else {
            false
        }; // TODO: Figure out if we need to use Option
        let repeat_alternative = if flags & 0x10 > 0 {
            read_repeat_alternative(io, &measure_headers)?
        } else {
            0
        };

        let marker = if flags & 0x20 > 0 {
            Some(read_marker(io)?)
        } else {
            None
        };

        let key_signature = if flags & 0x40 > 0 {
            let root = io.read_signed_byte()?;
            let signature_type = io.read_signed_byte()?;
            KeySignature {
                root: root,
                signature_type: signature_type,
            }
        } else if number > 1 {
            previous.key_signature
        } else {
            Default::default()
        };

        let has_double_bar = flags & 0x80 > 1;

        let measure_header = MeasureHeader {
            number: number,
            start: Duration::QuarterTime,
            time_signature: time_signature,
            key_signature: key_signature,
            tempo: song_tempo,
            triplet_feel: song_triplet_feel,
            is_repeat_open: is_repeat_open,
            repeat_close: repeat_close,
            repeat_alternative: repeat_alternative,
            real_start: -1, // TODO: Figure this out
            has_double_bar: has_double_bar,
            marker: marker,
            direction: None,
            from_direction: None,
        };
        previous = measure_header.clone();
        measure_headers.push(measure_header);
    }
    Ok(measure_headers)
}


// Measures are written in the following order:
// - measure 1/track 1
// - measure 1/track 2
// - ...
// - measure 1/track m
// - measure 2/track 1
// - measure 2/track 2
// - ...
// - measure 2/track m
// - ...
// - measure n/track 1
// - measure n/track 2
// - ...
// - measure n/track m
// fn read_measures<T>(io: &mut T, track_count: i32, channels: &mut [Channel]) -> Result<Vec<Track>>
//     where T: IoReader
// {
//     tempo = gp.Tempo(song.tempo)
//             start = gp.Duration.quarterTime
//             for header in song.measureHeaders:
//                 header.start = start
//                 for track in song.tracks:
//                     measure = gp.Measure(track, header)
//                     tempo = header.tempo
//                     track.measures.append(measure)
//                     self.readMeasure(measure)
//                 header.tempo = tempo
//                 start += header.length


// }

// The first byte is the track's flags. It presides the track's
// attributes:
// - *0x01*: drums track
// - *0x02*: 12 stringed guitar track
// - *0x04*: banjo track
// - *0x08*: *blank*
// - *0x10*: *blank*
// - *0x20*: *blank*
// - *0x40*: *blank*
// - *0x80*: *blank*
// Flags are followed by:
// - Name: :ref:`byte-size-string`. A 40 characters long string
//   containing the track's name.
// - Number of strings: :ref:`int`. An integer equal to the number
//     of strings of the track.
// - Tuning of the strings: List of 7 :ref:`Ints <int>`. The tuning
//   of the strings is stored as a 7-integers table, the "Number of
//   strings" first integers being really used. The strings are
//   stored from the highest to the lowest.
// - Port: :ref:`int`. The number of the MIDI port used.
// - Channel. See :meth:`GP3File.readChannel`.
// - Number of frets: :ref:`int`. The number of frets of the
//   instrument.
// - Height of the capo: :ref:`int`. The number of the fret on
//   which a capo is set. If no capo is used, the value is 0.
// - Track's color. The track's displayed color in Guitar Pro.
fn read_tracks<T>(io: &mut T, track_count: i32, channels: &mut [Channel]) -> Result<Vec<Track>>
    where T: IoReader
{
    let mut tracks = vec![];
    for number in 1..track_count + 1 {
        let flags = io.read_byte()?;
        let mut is_percussion_track = flags & 0x01 > 0;
        let is12_stringed_guitar_track = flags & 0x02 > 0;
        let is_banjo_track = flags & 0x04 > 0;
        let name = io.read_byte_sized_string(40)?;
        let string_count = io.read_int()?;
        let mut strings = vec![];
        for string_number in 1..8 {
            let tuning = io.read_int()?;
            if string_count >= string_number {
                let string = GuitarString { string_number, tuning };
                strings.push(string);
            }
        }
        let port = io.read_int()?;
        let channel_index = read_channel(io, channels)?;
        if channels[channel_index].channel == 9 {
            is_percussion_track = true; // Weird
        }
        let fret_count = io.read_int()?;
        let offeset = io.read_int()?;
        let color = read_color(io)?;
        let track = Track {
            number,
            is_percussion_track,
            is12_stringed_guitar_track,
            is_banjo_track,
            name,
            strings,
            port,
            channel_index,
            fret_count,
            offeset,
            color
        };
        tracks.push(track);
    }
    Ok(tracks)
}

// MIDI channel in Guitar Pro is represented by two integers. First
// is zero-based number of channel, second is zero-based number of
// channel used for effects.
fn read_channel<T>(io: &mut T, channels: &mut [Channel]) -> Result<usize>
    where T: IoReader
{
        let index = (io.read_int()? - 1) as usize;
        let effect_channel = io.read_int()? - 1;
        if 0 <= index && index < channels.len() {
            let track_channel = &mut channels[index];
            if track_channel.instrument < 0 {
                track_channel.instrument = 0;
            }
            if !track_channel.is_percussion_channel() {
                track_channel.effect_channel = effect_channel as u8;
            }
        }
        Ok(index)
}

// The markers are written in two steps. First is written an integer
// equal to the marker's name length + 1, then a string containing the
// marker's name. Finally the marker's color is written.

fn read_marker<T>(io: &mut T) -> Result<Marker>
    where T: IoReader
{
    let title = io.read_int_byte_sized_string()?;
    let color = read_color(io)?;
    Ok(Marker {
           title: title,
           color: color,
       })
}

// Colors are used by :class:`guitarpro.base.Marker` and
// :class:`guitarpro.base.Track`. They consist of 3 consecutive bytes and
// one blank byte.

fn read_color<T>(io: &mut T) -> Result<Color>
    where T: IoReader
{
    let r = io.read_byte()?;
    let g = io.read_byte()?;
    let b = io.read_byte()?;
    io.skip(1)?; //alpha? always 0x00
    Ok(Color { r: r, g: g, b: b })
}

fn read_repeat_alternative<T>(io: &mut T, measure_headers: &[MeasureHeader]) -> Result<u8>
    where T: IoReader
{
    let value = io.read_byte()?;
    let mut existing_alternatives = 0;
    for header in measure_headers.iter().rev() {
        if header.is_repeat_open {
            break;
        }
        existing_alternatives |= header.repeat_alternative;
    }
    Ok(1 << value - 1 ^ existing_alternatives)
}

fn to_channel_short(data: u8) -> i16 {
    let value = max(-32768i16, min(32767i16, ((data as i16) << 3) - 1));
    max(value, -1) + 1
}
