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

pub struct MeasureHeader;
pub struct Track;

pub struct Song {
    pub song_info: SongInfo,
    pub measure_headers: Vec<MeasureHeader>,
    pub tracks: Vec<Track>,
}
