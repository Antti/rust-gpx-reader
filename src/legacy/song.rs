use std::default::Default;

#[derive(Debug, Clone, Copy)]
pub enum TripletFeel {
    None = 0,
    Eighth = 1,
    Sixteenth = 2,
}

impl Default for TripletFeel {
    fn default() -> Self {
        TripletFeel::None
    }
}

#[derive(Debug)]
pub struct SongInfo {
    pub title: String,
    pub subtitle: String,
    pub artist: String,
    pub album: String,
    pub words: String,
    pub music: Option<String>, // Only in GP5
    pub copyright: String,
    pub tab: String,
    pub instructions: String,
    pub notice: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum DurationValue {
    QuarterTime = 960,
    Whole = 1,
    Half = 2,
    Quarter = 4,
    Eigth = 8,
    Sixteenth = 16,
    ThirtySecond = 32,
    SixtyFourth = 64,
    HundredTwentyEighth = 128
}

#[derive(Debug, Clone, Copy)]
pub struct Duration {
    pub value: DurationValue,
    pub is_dotted: bool,
    pub is_double_dotted: bool,
}

impl Default for Duration {
    fn default() -> Self {
        Self { value: DurationValue::QuarterTime, is_dotted: false, is_double_dotted: false }
    }
}

#[derive(Debug, Clone)]
pub struct TimeSignature {
    pub numerator: i8,
    pub denominator: i8, //TODO: Duration
    pub beams: Vec<u8>,
}

impl Default for TimeSignature {
    fn default() -> Self {
        TimeSignature {
            numerator: 0,
            denominator: 0,
            beams: vec![0, 0, 0, 0],
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct KeySignature {
    pub root: i8,
    pub signature_type: i8,
}

#[derive(Debug, Default, Clone)]
pub struct Marker {
    pub title: String,
    pub color: Color,
}

#[derive(Debug, Default, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Default, Clone)]
pub struct Direction;

#[derive(Debug, Default, Clone)]
pub struct MeasureHeader {
    pub number: u16,
    pub start: usize,
    pub time_signature: TimeSignature,
    pub key_signature: KeySignature,
    pub tempo: u16,
    pub triplet_feel: TripletFeel,
    pub is_repeat_open: bool,
    pub repeat_close: bool,
    pub repeat_alternative: u8,
    pub real_start: i16,
    pub has_double_bar: bool,
    pub marker: Option<Marker>,
    pub direction: Option<Direction>,
    pub from_direction: Option<Direction>,
}

pub struct Measure {
    pub header_index: usize,
    pub track_index: usize
}

pub enum MeasureClef {
    Trebble = 0,
    Bass = 1,
    Tenor = 2,
    Alto = 3
}

pub enum LineBreak {
    None = 0,
    Break = 1,
    Protect = 2
}

pub enum VoiceDirection {
    None = 0,
    Up = 1,
    Down = 2
}

#[derive(Debug)]
pub struct Track {
    pub number: i32,
    pub is_percussion_track: bool,
    pub is12_stringed_guitar_track: bool,
    pub is_banjo_track: bool,
    pub name: String,
    pub strings: Vec<GuitarString>,
    pub port: i32,
    pub channel_index: usize,
    pub fret_count: i32,
    pub offeset: i32,
    pub color: Color
}

#[derive(Debug)]
pub struct Song {
    pub song_info: SongInfo,
    pub triplet_feel: Option<TripletFeel>,
    pub tempo: i32,
    pub channels: Vec<Channel>,
    pub measure_headers: Vec<MeasureHeader>,
    pub tracks: Vec<Track>,
}

#[derive(Debug)]
pub struct GuitarString {
    pub string_number: i32,
    pub tuning: i32
}

#[derive(Debug)]
pub struct Lyrics {
    pub track: u32,
    pub lyrics: Vec<LyricsItem>
}

#[derive(Debug)]
pub struct LyricsItem {
    pub starting_measure: u32,
    pub text: String
}

#[derive(Debug)]
pub struct Channel {
    pub channel: u8,
    pub effect_channel: u8,
    pub instrument: i32,
    pub volume: i8,
    pub balance: i8,
    pub chorus: i8,
    pub reverb: i8,
    pub phaser: i8,
    pub tremolo: i8
}

const DEFAULT_PERCUSSION_CHANNEL: u8 = 9;

impl Channel {
    pub fn is_percussion_channel(&self) -> bool {
        (self.channel % 16 == DEFAULT_PERCUSSION_CHANNEL)
    }
}
