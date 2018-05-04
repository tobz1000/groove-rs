use c_api::{
    GroovePlaylistItem,
    GroovePlaylist,
    ANY_SINK_FULL,
    EVERY_SINK_FULL,
    groove_playlist_create,
    groove_playlist_destroy,
    groove_playlist_insert,
    groove_playlist_clear,
    groove_playlist_set_fill_mode,
};

use file::File;

pub struct PlaylistItem {
    groove_playlist_item: *mut GroovePlaylistItem,
    file: File
}

impl PlaylistItem {
    /// A volume adjustment in float format to apply to the file when it plays.
    /// This is typically used for loudness compensation, for example ReplayGain.
    /// To convert from dB to float, use exp(log(10) * 0.05 * dB_value)
    pub fn gain(&self) -> f64 {
        unsafe {
            (*self.groove_playlist_item).gain
        }
    }

    /// The sample peak of this playlist item is assumed to be 1.0 in float
    /// format. If you know for certain that the peak is less than 1.0, you
    /// may set this value which may allow the volume adjustment to use
    /// a pure amplifier rather than a compressor. This results in slightly
    /// better audio quality.
    pub fn peak(&self) -> f64 {
        unsafe {
            (*self.groove_playlist_item).peak
        }
    }

    pub fn file(&self) -> &File {
        &self.file
    }
}

/// a playlist keeps its sinks full.
pub struct Playlist {
    pub(crate) groove_playlist: *mut GroovePlaylist,
    items: Vec<PlaylistItem>
}

impl Drop for Playlist {
    fn drop(&mut self) {
        self.clear();
        unsafe { groove_playlist_destroy(self.groove_playlist) }
    }
}

impl Playlist {
    pub fn new() -> Self {
        super::init();
        unsafe {
            Playlist {
                groove_playlist: groove_playlist_create(),
                items: Vec::new()
            }
        }
    }

    /// volume adjustment in float format which applies to all playlist items
    /// and all sinks. defaults to 1.0.
    pub fn gain(&self) -> f64 {
        unsafe {
            (*self.groove_playlist).gain
        }
    }

    pub fn items(&self) -> &Vec<PlaylistItem> {
        &self.items
    }

    /// once you add a file to the playlist, you must not destroy it until you first
    /// remove it from the playlist.
    /// before: the item to insert before.
    /// gain: see Groove. use 1.0 for no adjustment.
    /// peak: see Groove. use 1.0 for no adjustment.
    /// returns the newly created playlist item.
    fn _insert(&mut self, file: File, gain: f64, peak: f64, index: Option<usize>) {
        let before_item = if let Some(index) = index {
            self.items[index].groove_playlist_item
        } else {
            ::std::ptr::null_mut()
        };

        let groove_playlist_item = unsafe {
            groove_playlist_insert(
                self.groove_playlist,
                file.groove_file,
                gain,
                peak,
                before_item
            )
        };

        if groove_playlist_item.is_null() {
            panic!("out of memory");
        }

        let playlist_item = PlaylistItem { groove_playlist_item, file };

        if let Some(index) = index {
            self.items.insert(index, playlist_item);
        } else {
            self.items.push(playlist_item);
        }
    }

    pub fn append(&mut self, file: File, gain: f64, peak: f64) {
        self._insert(file, gain, peak, None)
    }

    pub fn insert(&mut self, file: File, gain: f64, peak: f64, index: usize) {
        self._insert(file, gain, peak, Some(index))
    }

    /// remove all playlist items
    pub fn clear(&mut self) {
        unsafe {
            groove_playlist_clear(self.groove_playlist);
            self.items.clear();
        }
    }

    pub fn set_fill_mode(&self, mode: FillMode) {
        let mode_int = match mode {
            FillMode::EverySinkFull => EVERY_SINK_FULL,
            FillMode::AnySinkFull   => ANY_SINK_FULL,
        };

        unsafe { groove_playlist_set_fill_mode(self.groove_playlist, mode_int) }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum FillMode {
    /// This is the default behavior. The playlist will decode audio if any sinks
    /// are not full. If any sinks do not drain fast enough the data will buffer up
    /// in the playlist.
    EverySinkFull,

    /// With this behavior, the playlist will stop decoding audio when any attached
    /// sink is full, and then resume decoding audio every sink is not full.
    AnySinkFull,
}