use c_api::{
    GroovePlaylistItem,
    GroovePlaylist,
    GrooveFile,
    ANY_SINK_FULL,
    EVERY_SINK_FULL,
    groove_playlist_create,
    groove_playlist_destroy,
    groove_playlist_insert,
    groove_playlist_count,
    groove_playlist_clear,
    groove_playlist_set_fill_mode,
};

use super::GROOVE_FILE_RC;
use file::File;

pub struct PlaylistItem {
    groove_playlist_item: *mut GroovePlaylistItem,
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

    pub fn file(&self) -> File {
        unsafe {
            let groove_file = (*self.groove_playlist_item).file;
            GROOVE_FILE_RC.lock().unwrap().incr(groove_file);
            File {groove_file: groove_file}
        }
    }
}

/// a playlist keeps its sinks full.
pub struct Playlist {
    pub(crate) groove_playlist: *mut GroovePlaylist,
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
            Playlist { groove_playlist: groove_playlist_create() }
        }
    }

    /// volume adjustment in float format which applies to all playlist items
    /// and all sinks. defaults to 1.0.
    pub fn gain(&self) -> f64 {
        unsafe {
            (*self.groove_playlist).gain
        }
    }

    /// get the first playlist item
    pub fn first(&self) -> PlaylistItem {
        unsafe {
            PlaylistItem {groove_playlist_item: (*self.groove_playlist).head }
        }
    }

    /// get the last playlist item
    pub fn last(&self) -> PlaylistItem {
        unsafe {
            PlaylistItem {groove_playlist_item: (*self.groove_playlist).tail }
        }
    }

    pub fn iter(&self) -> PlaylistIterator {
        unsafe {
            PlaylistIterator { curr: (*self.groove_playlist).head }
        }
    }

    /// once you add a file to the playlist, you must not destroy it until you first
    /// remove it from the playlist.
    /// gain: see PlaylistItem. use 1.0 for no adjustment.
    /// peak: see PlaylistItem. use 1.0 for no adjustment.
    /// returns the newly created playlist item.
    pub fn append(&self, file: &File, gain: f64, peak: f64) -> PlaylistItem {
        unsafe {
            let inserted_item = groove_playlist_insert(self.groove_playlist, file.groove_file,
                                                       gain, peak, ::std::ptr::null_mut());
            if inserted_item.is_null() {
                panic!("out of memory");
            } else {
                GROOVE_FILE_RC.lock().unwrap().incr(file.groove_file);
                PlaylistItem {groove_playlist_item: inserted_item}
            }
        }
    }

    /// once you add a file to the playlist, you must not destroy it until you first
    /// remove it from the playlist.
    /// before: the item to insert before.
    /// gain: see Groove. use 1.0 for no adjustment.
    /// peak: see Groove. use 1.0 for no adjustment.
    /// returns the newly created playlist item.
    pub fn insert(&self, file: &File, gain: f64, peak: f64, before: &PlaylistItem) -> PlaylistItem {
        unsafe {
            let inserted_item = groove_playlist_insert(self.groove_playlist, file.groove_file,
                                                       gain, peak, before.groove_playlist_item);
            if inserted_item.is_null() {
                panic!("out of memory");
            } else {
                GROOVE_FILE_RC.lock().unwrap().incr(file.groove_file);
                PlaylistItem {groove_playlist_item: inserted_item}
            }
        }
    }

    /// return the count of playlist items
    pub fn len(&self) -> i32 {
        unsafe {
            groove_playlist_count(self.groove_playlist) as i32
        }
    }

    /// remove all playlist items
    pub fn clear(&self) {
        unsafe {
            let groove_files: Vec<*mut GrooveFile> =
                self.iter().map(|x| (*x.groove_playlist_item).file).collect();
            groove_playlist_clear(self.groove_playlist);
            for groove_file in groove_files.iter() {
                GROOVE_FILE_RC.lock().unwrap().decr(*groove_file);
            }
        }
    }

    pub fn set_fill_mode(&self, mode: FillMode) {
        let mode_int = match mode {
            FillMode::EverySinkFull => EVERY_SINK_FULL,
            FillMode::AnySinkFull   => ANY_SINK_FULL,
        };
        unsafe { groove_playlist_set_fill_mode(self.groove_playlist, mode_int) }
    }

    fn get_groove_playlist(&self) -> *mut GroovePlaylist {
        self.groove_playlist
    }
}

pub struct PlaylistIterator {
    curr: *mut GroovePlaylistItem,
}

impl Iterator for PlaylistIterator {
    type Item = PlaylistItem;

    fn next(&mut self) -> Option<PlaylistItem> {
        unsafe {
            if self.curr.is_null() {
                Option::None
            } else {
                let prev = self.curr;
                self.curr = (*self.curr).next;
                Option::Some(PlaylistItem {groove_playlist_item: prev})
            }
        }
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