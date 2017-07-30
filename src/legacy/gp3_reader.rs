use super::io_reader::IoReader;
use super::super::Result;
use super::song::*;

use std::default::Default;

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
            Duration { value: io.read_signed_byte()?.into(), .. Default::default() }
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
            start: DurationValue::QuarterTime as usize,
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
// Measure track pairs
fn read_measures<T>(io: &mut T, tracks: &mut [Track], measureHeaders: &mut [MeasureHeader], tempo: u16) -> Result<Vec<Measure>>
    where T: IoReader
{
    let mut start = DurationValue::QuarterTime as usize;
    let mut measures = vec![];
    for (measure_index, header) in measureHeaders.iter_mut().enumerate() {
        header.start = start;
        for (track_index, track) in tracks.iter_mut().enumerate() {
            let number_of_beats = io.read_int()?;
            let measure = Measure { track_index, measure_index }; // ?
            for b in 0..number_of_beats {
                // reading beat
                let beat =  read_beat(io);
                start += 0;
            }
            track.measures.push(measure);
            // tempo = header.tempo
        }
        // header.tempo = tempo
        start += header.time_signature.len()
    }
    Ok(measures)
}


// The first byte is the beat flags. It lists the data present in
// the current beat:
// - *0x01*: dotted notes
// - *0x02*: presence of a chord diagram
// - *0x04*: presence of a text
// - *0x08*: presence of effects
// - *0x10*: presence of a mix table change event
// - *0x20*: the beat is a n-tuplet
// - *0x40*: status: True if the beat is empty of if it is a rest
// - *0x80*: *blank*
// Flags are followed by:
// - Status: :ref:`byte`. If flag at *0x40* is true, read one byte.
//     If value of the byte is *0x00* then beat is empty, if value is
//     *0x02* then the beat is rest.
// - Beat duration: :ref:`byte`. See :meth:`readDuration`.
// - Chord diagram. See :meth:`readChord`.
// - Text. See :meth:`readText`.
// - Beat effects. See :meth:`readBeatEffects`.
// - Mix table change effect. See :meth:`readMixTableChange`.
pub fn read_beat<T>(io: &mut T) -> Result<Beat>
    where T: IoReader
{
    let flags = io.read_byte()?;
    let status = if flags & 0x40 > 0 {
        io.read_byte()?.into()
    } else {
        BeatStatus::Normal
    };
    let duration = read_duration(io, flags)?;
    if flags & 0x02 > 0 {
        // read chord
    }
    if flags & 0x04 > 0 {
        // read text
    }
    if flags & 0x08 > 0 {
        // read beat effects
    }
    if flags & 0x10 > 0 {
        // read mix table change
    }

    Ok(Beat {
        notes: vec![],
        duration: duration,
        text: String::from(""),
        start: 0,
        effect: BeatEffect,
        index: 0,
        octave: Octave,
        display: BeatDisplay,
        status: BeatStatus::Empty
    })

    // read notes

    // duration = self.readDuration(flags)
    // effect = gp.NoteEffect()
    // if flags & 0x02:
    //     beat.effect.chord = self.readChord(len(voice.measure.track.strings))
    // if flags & 0x04:
    //     beat.text = self.readText()
    // if flags & 0x08:
    //     beat.effect = self.readBeatEffects(effect)
    // if flags & 0x10:
    //     mixTableChange = self.readMixTableChange(voice.measure)
    //     beat.effect.mixTableChange = mixTableChange
    // self.readNotes(voice.measure.track, beat, duration, effect)
    // return duration.time if not beat.status == gp.BeatStatus.empty else 0
}


// Duration is composed of byte signifying duration and an integer
// that maps to :class:`guitarpro.models.Tuplet`.
// The byte maps to following values:
// - *-2*: whole note
// - *-1*: half note
// -  *0*: quarter note
// -  *1*: eighth note
// -  *2*: sixteenth note
// -  *3*: thirty-second note
// If flag at *0x20* is true, the tuplet is read.

pub fn read_duration<T>(io: &mut T, flags: u8) -> Result<Duration>
    where T: IoReader
{
    let value = 1 << (io.read_signed_byte()? + 2);
    let is_dotted = flags & 0x01 > 0;
    let tuplet = if flags & 0x20 > 0 {
        let tuplet_int = io.read_int()?;
        match tuplet_int {
            3 => Tuplet { enters: 3, times: 2 },
            5 => Tuplet { enters: 5, times: 4 },
            6 => Tuplet { enters: 6, times: 4 },
            7 => Tuplet { enters: 7, times: 4 },
            9 => Tuplet { enters: 9, times: 8 },
            10 => Tuplet { enters: 10, times: 8 },
            11 => Tuplet { enters: 11, times: 8 },
            12 => Tuplet { enters: 12, times: 8 },
            _ => panic!("Unexpeced tuplet number")
        }
    } else {
        Default::default()
    };
    // TODO: Check how is_double_dotted read.
    Ok(Duration { value: value.into(), is_dotted, tuplet, is_double_dotted: false })
}

// First byte is chord header. If it's set to 0, then following
// chord is written in default (GP3) format. If chord header is set
// to 1, then chord diagram in encoded in more advanced (GP4)
// format.
pub fn read_chord<T>(io: &mut T, strings_count: u8) -> Result<Chord>
    where T: IoReader
{
    let new_format = io.read_bool()?;
    if new_format {
        let chord = read_new_chord(io)?;
        Ok(Chord::NewChord(chord))
    } else {
        let mut chord = read_old_chord(io)?;
        chord.frets.truncate(strings_count as usize);
        Ok(Chord::OldChord(chord))
    }
}

// Read new-style (GP4) chord diagram.
// New-style chord diagram is read as follows:
// - Sharp: :ref:`bool`. If true, display all semitones as sharps,
//     otherwise display as flats.
// - Blank space, 3 :ref:`Bytes <byte>`.
// - Root: :ref:`int`. Values are:
//     * -1 for customized chords
//     *  0: C
//     *  1: C#
//     * ...
// - Type: :ref:`int`. Determines the chord type as followed. See
//     :class:`guitarpro.models.ChordType` for mapping.
// - Chord extension: :ref:`int`. See
//     :class:`guitarpro.models.ChordExtension` for mapping.
// - Bass note: :ref:`int`. Lowest note of chord as in *C/Am*.
// - Tonality: :ref:`int`. See
//     :class:`guitarpro.models.ChordAlteration` for mapping.
// - Add: :ref:`bool`. Determines if an "add" (added note) is
//     present in the chord.
// - Name: :ref:`byte-size-string`. Max length is 22.
// - Fifth alteration: :ref:`int`. Maps to
//     :class:`guitarpro.models.ChordAlteration`.
// - Ninth alteration: :ref:`int`. Maps to
//     :class:`guitarpro.models.ChordAlteration`.
// - Eleventh alteration: :ref:`int`. Maps to
//     :class:`guitarpro.models.ChordAlteration`.
// - List of frets: 6 :ref:`Ints <int>`. Fret values are saved as
//     in default format.
// - Count of barres: :ref:`int`. Maximum count is 2.
// - Barre frets: 2 :ref:`Ints <int>`.
// - Barre start strings: 2 :ref:`Ints <int>`.
// - Barre end string: 2 :ref:`Ints <int>`.
// - Omissions: 7 :ref:`Bools <bool>`. If the value is true then
//     note is played in chord.
// - Blank space, 1 :ref:`byte`.

pub fn read_new_chord<T>(io: &mut T) -> Result<NewChord>
    where T: IoReader
{
//    Ok(NewChord { frets: vec![], first_fret: 0 })
    loop {}
}

// Read chord diagram encoded in GP3 format.
// Chord diagram is read as follows:
// - Name: :ref:`int-byte-size-string`. Name of the chord, e.g.
//     *Em*.
// - First fret: :ref:`int`. The fret from which the chord is
//     displayed in chord editor.
// - List of frets: 6 :ref:`Ints <int>`. Frets are listed in order:
//     fret on the string 1, fret on the string 2, ..., fret on the
//     string 6. If string is untouched then the values of fret is
//     *-1*.
pub fn read_old_chord<T>(io: &mut T) -> Result<OldChord>
    where T: IoReader
{
    let name = io.read_int_byte_sized_string()?;
    let first_fret = io.read_int()?;
    let mut frets = vec![];
    if first_fret > 0 {
        for i in 0..6 { // always read 6 ints
            frets.push(io.read_int()?);
        }
    }
    Ok(OldChord { frets, first_fret })
}

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
            color,
            measures: vec![]
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
        if index < channels.len() { //sanity check?
            let track_channel = &mut channels[index];
            if track_channel.instrument < 0 {
                track_channel.instrument = 0;
            }
            if !track_channel.is_percussion_channel() {
                track_channel.effect_channel = effect_channel as u8;
            }
        } else {
            panic!("channel index is {}", index);
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
    Ok(Marker { title, color })
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
    Ok(Color { r, g, b })
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

// TODO: Do we need this?
// fn to_channel_short(data: u8) -> i16 {
//     use std::cmp::{max, min};
//     let value = max(-32768i16, min(32767i16, ((data as i16) << 3) - 1));
//     max(value, -1) + 1
// }
