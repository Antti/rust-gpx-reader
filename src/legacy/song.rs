use std::default::Default;

#[derive(Debug)]
pub enum TripletFeel {
    None,
    Eighth,
    Sixteenth
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
    pub notice: Vec<String>
}

#[derive(Debug)]
pub enum Duration {
    QuarterTime
}

impl Default for Duration {
    fn default() -> Self {
        Duration::QuarterTime
    }
}


#[derive(Debug)]
pub struct TimeSignature {
    pub numerator: i8,
    pub denominator: i8, //TODO: Duration
    pub beams: Vec<u8>
}

impl Default for TimeSignature {
    fn default() -> Self {
        TimeSignature { numerator: 0, denominator: 0, beams: vec![0,0,0,0] }
    }
}

#[derive(Debug, Default)]
pub struct KeySignature;
//
// impl Default for TimeSignature {
//     fn default() -> Self {
//         TimeSignature { numerator: 0, denominator: 0, beams: vec![] }
//     }
// }

#[derive(Debug, Default)]
pub struct MeasureHeader {
    pub number: u16,
    pub start: Duration,
    pub time_signature: TimeSignature,
    pub key_signature: KeySignature,
    pub tempo: u16,
    pub triplet_feel: TripletFeel,
    pub is_repeat_open: bool,
    pub repeat_close: i16,
    pub repeat_alternative: u16,
    pub real_start: i16,
    pub has_double_bar: bool
}

#[derive(Debug)]
pub struct Track;

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
    fn is_percussion_channel(&self) -> bool {
        (self.channel % 16 == DEFAULT_PERCUSSION_CHANNEL)
    }
}
