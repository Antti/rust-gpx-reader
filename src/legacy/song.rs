use std::default::Default;
use std::convert::From;
use enum_primitive::FromPrimitive;

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

enum_from_primitive! {
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
}

impl From<u16> for DurationValue {
    fn from(value: u16) -> DurationValue {
        DurationValue::from_u16(value).expect("Unknown duration")
    }
}

impl From<i8> for DurationValue {
    fn from(value: i8) -> DurationValue {
        DurationValue::from_i8(value).expect("Unknown duration")
    }
}

impl From<i32> for DurationValue {
    fn from(value: i32) -> DurationValue {
        DurationValue::from_i32(value).expect("Unknown duration")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Duration {
    pub value: DurationValue,
    pub is_dotted: bool,
    pub is_double_dotted: bool,
    pub tuplet: Tuplet
}

impl Default for Duration {
    fn default() -> Self {
        let tuplet = Default::default();
        Self { value: DurationValue::QuarterTime, is_dotted: false, is_double_dotted: false, tuplet }
    }
}

impl Duration {
    pub fn time(&self) -> usize {
        let mut result = (DurationValue::QuarterTime as usize as f64 * (4.0 / self.value as usize as f64)) as usize;
        if self.is_dotted {
            result += result / 2;
        }
        if self.is_double_dotted {
            result += result / 4 * 3;
        }
        result * self.tuplet.times as usize / self.tuplet.enters as usize
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tuplet {
    pub enters: u8,
    pub times: u8
}

impl Default for Tuplet {
    fn default() -> Self {
        Tuplet { enters: 1, times: 1 }
    }
}

#[derive(Debug, Clone)]
pub struct TimeSignature {
    pub numerator: i8,
    pub denominator: Duration,
    pub beams: Vec<u8>,
}

impl Default for TimeSignature {
    fn default() -> Self {
        TimeSignature {
            numerator: 0,
            denominator: Default::default(),
            beams: vec![0, 0, 0, 0],
        }
    }
}

impl TimeSignature {
    pub fn len(&self) -> usize {
        self.numerator as usize * self.denominator.time()
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

// Actual measure
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

// is actually a measure/track pair
#[derive(Debug, Default, Clone)]
pub struct Measure {
    pub measure_index: usize,
    pub track_index: usize
}

#[derive(Debug, Clone)]
pub struct Beat {
    pub notes: Vec<Note>,
    pub duration: Duration,
    pub text: String,
    pub start: usize,
    pub effect: BeatEffect,
    pub index: usize,
    pub octave: Octave,
    pub display: Option<BeatDisplay>, // Not GP3
    pub status: BeatStatus
}

impl Beat {
    pub fn time(&self) -> usize {
        match self.status {
            BeatStatus::Empty => 0,
            _ => self.duration.time()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BeatEffect {
    pub stroke: BeatStroke,
    pub has_rasgueado: bool,
    pub pick_stroke: Option<BeatStrokeDirection>,
    pub chord: Option<Chord>,
    pub fade_in: bool,
    pub tremolo_bar: Option<BendEffect>,
    pub mix_table_change: Option<MixTableChange>,
    pub slap_effect: Option<SlapEffect>,
    pub vibrato: Option<Vibrato>
}

#[derive(Debug, Clone)]
pub struct BeatDisplay {
    pub break_beam: bool,
    pub force_beam: bool,
    pub beam_direction: VoiceDirection,
    pub tuplet_bracket: TupletBracket,
    pub break_secondary: u8,
    pub break_secondary_tuplet: bool,
    pub force_bracket: bool
}

#[derive(Debug, Clone)]
pub enum TupletBracket {
    None = 0,
    Start = 1,
    End = 2
}

#[derive(Debug, Clone, Default)]
pub struct BeatStroke {
    pub direction: Option<BeatStrokeDirection>,
    pub value: u8
}

#[derive(Debug, Clone)]
pub enum BeatStrokeDirection {
    Up,
    Down
}

#[derive(Debug, Clone)]
pub struct TremoloBar;

#[derive(Debug, Clone)]
pub struct MixTableChange;

#[derive(Debug, Clone)]
pub enum SlapEffect {
    None = 0,
    Tapping = 1,
    Slapping = 2,
    Popping = 3
}

#[derive(Debug, Clone)]
pub struct Vibrato;

enum_from_primitive! {
#[derive(Debug, Clone)]
pub enum BeatStatus {
    Empty = 0,
    Normal = 1,
    Rest = 2
}
}

#[derive(Debug, Clone)]
pub struct BendEffect {
    pub effect_type: BendType,
    pub value: u8,
    pub points: Vec<BendPoint>
}

#[derive(Debug, Clone)]
pub struct BendPoint {
    pub position: u8,
    pub value: u8,
    pub vibrato: bool
}

enum_from_primitive! {
#[derive(Debug, Clone)]
pub enum BendType {
    // No Preset.
    none = 0,
    // A simple bend.
    bend = 1,
    // A bend and release afterwards.
    bend_release = 2,
    // A bend, then release and rebend.
    bend_release_bend = 3,
    // Prebend.
    prebend = 4,
    // Prebend and then release.
    prebend_release = 5,
    // Tremolo Bar
    //
    // Dip the bar down and then back up.
    dip = 6,
    // Dive the bar.
    dive = 7,
    // Release the bar up.
    release_up = 8,
    // Dip the bar up and then back down.
    inverted_dip = 9,
    // Return the bar.
    return_ = 10,
    // Release the bar down.
    release_down = 11
}
}

impl From<u8> for BeatStatus {
    fn from(value: u8) -> BeatStatus {
        BeatStatus::from_u8(value).expect("Unknown beat status")
    }
}

#[derive(Debug, Clone)]
pub struct Voice {
    pub beats: Vec<Beat>
}

#[derive(Debug, Clone)]
pub struct Note {
    pub beat: Beat,
    pub value: u8,
    pub velocity: Velocity,
    pub string: u8,
    pub effect: NoteEffect,
    pub duration_percent: f32,
    pub swap_accidentals: bool,
    pub note_type: NoteType
}

#[derive(Debug, Clone)]
pub struct NoteEffect {
    pub accentuated_note: bool,
    pub bend: Option<BendEffect>,
    pub ghost_note: bool,
    pub grace: Option<GraceEffect>,
    pub hammer: bool,
    pub harmonic: Option<HarmonicEffect>,
    pub heavy_accentuated_note: bool,
    pub left_hand_finger: Fingering,
    pub let_ring: bool,
    pub palm_mute: bool,
    pub right_hand_finger: Fingering,
    pub slides: Vec<SlideType>,
    pub staccato: bool,
    pub tremolo_picking: Option<TremoloPickingEffect>,
    pub trill: Option<TrillEffect>,
    pub vibrato: bool
}

enum_from_primitive! {
#[derive(Debug, Clone)]
pub enum NoteType {
    Rest = 0,
    Normal = 1,
    Tie = 2,
    Dead = 3
}
}

#[derive(Debug, Clone)]
pub struct GraceEffect {
    pub duration: u8,
    pub fret: u8,
    pub is_dead: bool,
    pub is_on_beat: bool,
    pub transition: GraceEffectTransition,
    pub velocity: Velocity
}

#[derive(Debug, Clone)]
pub enum HarmonicEffect {
    NaturalHarmonic,
    ArtificialHarmonic(u8, Octave),
    TappedHarmonic(u8),
    PinchHarmonic,
    SemiHarmonic
}

#[derive(Debug, Clone)]
pub struct TremoloPickingEffect {
    pub duration: Duration
}

#[derive(Debug, Clone)]
pub struct TrillEffect {
    pub fret: u8,
    pub duration: Duration
}

#[derive(Debug, Clone)]
pub enum Velocity {
    Default
}

enum_from_primitive! {
#[derive(Debug, Clone)]
pub enum Fingering {
    Unknown = -2,
    Open = -1,
    Thumb = 0,
    Index = 1,
    Middle = 2,
    Annular = 3,
    Little = 4
}
}

enum_from_primitive! {
#[derive(Debug, Clone)]
pub enum SlideType {
    into_from_above = -2,
    into_from_below = -1,
    none = 0,
    shift_slide_to = 1,
    legato_slide_to = 2,
    out_downwards = 3,
    out_upwards = 4
}
}

enum_from_primitive! {
#[derive(Debug, Clone)]
pub enum GraceEffectTransition {
    None = 0,
    // Slide from the grace note to the real one.
    Slide = 1,
    // Perform a bend from the grace note to the real one.
    Bend = 2,
    // Perform a hammer on.
    Hammer = 3
}
}



#[derive(Debug, Clone)]
pub enum Octave {
    None = 0,
    Ottava = 1,
    Quindicesima = 2,
    OttavaBassa = 3,
    QuindicesimaBassa = 4
}

#[derive(Debug, Clone)]
pub struct OldChord {
    pub first_fret: i32,
    pub frets: Vec<i32>
}

#[derive(Debug, Clone)]
pub struct NewChord {
    pub length: usize,
    pub sharp: bool,
    pub root: bool,
    pub chord_type: ChordType,
    pub extension: bool,
    pub bass: bool,
    pub tonality: bool,
    pub add: bool,
    pub name: String,
    pub fifth: bool,
    pub ninth: bool,
    pub eleventh: bool,
    pub first_fret: bool,
    pub strings: Vec<u8>,
    pub barres: Vec<Barre>,
    pub omissions: Vec<u8>,
    pub fingerings: Vec<u8>,
    pub show: bool,
}

#[derive(Debug, Clone)]
pub enum Chord {
    NewChord(NewChord),
    OldChord(OldChord)
}

#[derive(Debug, Clone)]
pub enum ChordType {
    // Major chord.
    Major = 0,
    // Dominant seventh chord.
    Seventh = 1,
    // Major seventh chord.
    MajorSeventh = 2,
    // Add sixth chord.
    Sixth = 3,
    // Minor chord.
    Minor = 4,
    // Minor seventh chord.
    MinorSeventh = 5,
    // Minor major seventh chord.
    MinorMajor = 6,
    // Minor add sixth chord.
    MinorSixth = 7,
    // Suspended second chord.
    SuspendedSecond = 8,
    // Suspended fourth chord.
    SuspendedFourth = 9,
    // Seventh suspended second chord.
    SeventhSuspendedSecond = 10,
    // Seventh suspended fourth chord.
    SeventhSuspendedFourth = 11,
    // Diminished chord.
    Diminished = 12,
    // Augmented chord.
    Augmented = 13,
    // Power chord.
    Power = 14
}

#[derive(Debug, Clone)]
pub enum ChordAlteration {
    Perfect = 0,
    Diminished = 1,
    Augmented = 2
}

#[derive(Debug, Clone)]
pub enum ChordExtension {
    None = 0,
    Ninth = 1,
    Eleventh = 2,
    Thirteenth = 3
}

#[derive(Debug, Clone)]
pub struct Barre {
    pub start: u8,
    pub end: u8
}

#[derive(Debug, Clone)]
pub enum MeasureClef {
    Trebble = 0,
    Bass = 1,
    Tenor = 2,
    Alto = 3
}

#[derive(Debug, Clone)]
pub enum LineBreak {
    None = 0,
    Break = 1,
    Protect = 2
}

#[derive(Debug, Clone)]
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
    pub color: Color,
    pub measures: Vec<Measure>
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
