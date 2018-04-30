extern crate libc;

use libc::{c_int, uint64_t, c_char, c_void, c_double, uint8_t};

#[repr(C)]
pub struct GrooveSink {
    pub audio_format: GrooveAudioFormat,
    pub disable_resample: c_int,
    /// If you leave this to its default of 0, frames pulled from the sink
    /// will have sample count determined by efficiency.
    /// If you set this to a positive number, frames pulled from the sink
    /// will always have this number of samples.
    pub buffer_sample_count: c_int,

    /// how big the buffer queue should be, in sample frames.
    /// groove_sink_create defaults this to 8192
    pub buffer_size: c_int,

    /// This volume adjustment only applies to this sink.
    /// It is recommended that you leave this at 1.0 and instead adjust the
    /// gain of the playlist.
    /// If you want to change this value after you have already attached the
    /// sink to the playlist, you must use groove_sink_set_gain.
    /// float format. Defaults to 1.0
    pub gain: c_double,

    /// set to whatever you want
    pub userdata: *mut c_void,
    /// called when the audio queue is flushed. For example, if you seek to a
    /// different location in the song.
    pub flush: extern fn(sink: *mut GrooveSink),
    /// called when a playlist item is deleted. Take this opportunity to remove
    /// all your references to the GroovePlaylistItem.
    pub purge: extern fn(sink: *mut GrooveSink, item: *mut GroovePlaylistItem),
    /// called when the playlist is paused
    pub pause: extern fn(sink: *mut GrooveSink),
    /// called when the playlist is played
    pub play: extern fn(sink: *mut GrooveSink),

    /// read-only. set when you call groove_sink_attach. cleared when you call
    /// groove_sink_detach
    pub playlist: *mut GroovePlaylist,

    /// read-only. automatically computed from audio_format when you call
    /// groove_sink_attach
    pub bytes_per_sec: c_int,
}

/// all fields read-only
#[repr(C)]
pub struct GrooveBuffer {
    /// for interleaved audio, data[0] is the buffer.
    /// for planar audio, each channel has a separate data pointer.
    /// for encoded audio, data[0] is the encoded buffer.
    pub data: *mut *mut uint8_t,

    pub format: GrooveAudioFormat,

    /// number of audio frames described by this buffer
    /// for encoded audio, this is unknown and set to 0.
    pub frame_count: c_int,

    /// when encoding, if item is NULL, this is a format header or trailer.
    /// otherwise, this is encoded audio for the item specified.
    /// when decoding, item is never NULL.
    pub item: *mut GroovePlaylistItem,
    pub pos: c_double,

    /// total number of bytes contained in this buffer
    pub size: c_int,

    /// presentation time stamp of the buffer
    pub pts: uint64_t,
}
// Read-only structs are Sync
unsafe impl Sync for GrooveBuffer {}
// Promise rust that nothing points to a GrooveBuffer
// when it destructs
unsafe impl Send for GrooveBuffer {}

/// all fields are read-only. modify with methods
#[repr(C)]
pub struct GroovePlaylistItem {
    pub file: *mut GrooveFile,

    pub gain: c_double,
    pub peak: c_double,

    /// A GroovePlaylist is a doubly linked list. Use these fields to
    /// traverse the list.
    pub prev: *mut GroovePlaylistItem,
    pub next: *mut GroovePlaylistItem,
}

/// a GroovePlaylist keeps its sinks full.
/// all fields are read-only. modify using methods.
#[repr(C)]
pub struct GroovePlaylist {
    /// doubly linked list which is the playlist
    pub head: *mut GroovePlaylistItem,
    pub tail: *mut GroovePlaylistItem,

    pub gain: c_double,
}

#[repr(C)]
pub struct GrooveFile {
    pub dirty: c_int,
    pub filename: *const c_char,
}

unsafe impl Send for GrooveFile {}

#[repr(C)]
pub struct GrooveAudioFormat {
    pub sample_rate: c_int,
    pub channel_layout: uint64_t,
    pub sample_fmt: c_int,
}

#[repr(C)]
pub struct GrooveEncoder {
    pub target_audio_format: GrooveAudioFormat,
    pub bit_rate: c_int,
    pub format_short_name: *const c_char,
    pub codec_short_name: *const c_char,
    pub filename: *const c_char,
    pub mime_type: *const c_char,

    /// how big the sink buffer should be, in sample frames.
    /// groove_encoder_create defaults this to 8192
    pub sink_buffer_size: c_int,

    /// how big the encoded audio buffer should be, in bytes
    /// groove_encoder_create defaults this to 16384
    pub encoded_buffer_size: c_int,

    /// This volume adjustment to make to this player.
    /// It is recommended that you leave this at 1.0 and instead adjust the
    /// gain of the underlying playlist.
    /// If you want to change this value after you have already attached the
    /// sink to the playlist, you must use groove_encoder_set_gain.
    /// float format. Defaults to 1.0
    pub gain: c_double,

    /// read-only. set when attached and cleared when detached
    pub playlist: *mut GroovePlaylist,

    pub actual_audio_format: GrooveAudioFormat,
}

pub const EVERY_SINK_FULL: c_int = 0;
pub const ANY_SINK_FULL:   c_int = 1;

pub const TAG_MATCH_CASE: c_int = 1;

pub const BUFFER_NO:  c_int = 0;
pub const BUFFER_YES: c_int = 1;
pub const BUFFER_END: c_int = 2;

pub const CH_FRONT_LEFT    :uint64_t = 0x00000001;
pub const CH_FRONT_RIGHT   :uint64_t = 0x00000002;
pub const CH_FRONT_CENTER  :uint64_t = 0x00000004;
pub const CH_LAYOUT_MONO   :uint64_t = CH_FRONT_CENTER;
pub const CH_LAYOUT_STEREO :uint64_t = CH_FRONT_LEFT|CH_FRONT_RIGHT;

pub const SAMPLE_FMT_NONE: i32 = -1;
pub const SAMPLE_FMT_U8:   i32 =  0;
pub const SAMPLE_FMT_S16:  i32 =  1;
pub const SAMPLE_FMT_S32:  i32 =  2;
pub const SAMPLE_FMT_FLT:  i32 =  3;
pub const SAMPLE_FMT_DBL:  i32 =  4;

pub const SAMPLE_FMT_U8P:  i32 =  5;
pub const SAMPLE_FMT_S16P: i32 =  6;
pub const SAMPLE_FMT_S32P: i32 =  7;
pub const SAMPLE_FMT_FLTP: i32 =  8;
pub const SAMPLE_FMT_DBLP: i32 =  9;

#[link(name="groove")]
extern {
    pub fn groove_init() -> c_int;
    pub fn groove_finish();
    pub fn groove_set_logging(level: c_int);
    pub fn groove_channel_layout_count(channel_layout: uint64_t) -> c_int;
    pub fn groove_channel_layout_default(count: c_int) -> uint64_t;
    pub fn groove_sample_format_bytes_per_sample(format: c_int) -> c_int;
    pub fn groove_version_major() -> c_int;
    pub fn groove_version_minor() -> c_int;
    pub fn groove_version_patch() -> c_int;
    pub fn groove_version() -> *const c_char;

    pub fn groove_file_open(filename: *const c_char) -> *mut GrooveFile;
    pub fn groove_file_close(file: *mut GrooveFile);
    pub fn groove_file_duration(file: *mut GrooveFile) -> c_double;
    pub fn groove_file_metadata_get(file: *mut GrooveFile, key: *const c_char,
                                prev: *const c_void, flags: c_int) -> *mut c_void;
    pub fn groove_file_metadata_set(file: *mut GrooveFile, key: *const c_char,
                                value: *const c_char, flags: c_int) -> c_int;
    pub fn groove_file_save(file: *mut GrooveFile) -> c_int;
    pub fn groove_file_audio_format(file: *mut GrooveFile, audio_format: *mut GrooveAudioFormat);

    pub fn groove_tag_key(tag: *mut c_void) -> *const c_char;
    pub fn groove_tag_value(tag: *mut c_void) -> *const c_char;

    pub fn groove_playlist_create() -> *mut GroovePlaylist;
    pub fn groove_playlist_insert(playlist: *mut GroovePlaylist, file: *mut GrooveFile,
                              gain: c_double, peak: c_double,
                              next: *mut GroovePlaylistItem) -> *mut GroovePlaylistItem;
    pub fn groove_playlist_destroy(playlist: *mut GroovePlaylist);
    pub fn groove_playlist_count(playlist: *mut GroovePlaylist) -> c_int;
    pub fn groove_playlist_clear(playlist: *mut GroovePlaylist);
    pub fn groove_playlist_set_fill_mode(playlist: *mut GroovePlaylist, mode: c_int);

    pub fn groove_encoder_create() -> *mut GrooveEncoder;
    pub fn groove_encoder_destroy(encoder: *mut GrooveEncoder);
    pub fn groove_encoder_metadata_set(encoder: *mut GrooveEncoder, key: *const c_char,
                                   value: *const c_char, flags: c_int) -> c_int;
    pub fn groove_encoder_attach(encoder: *mut GrooveEncoder, playlist: *mut GroovePlaylist) -> c_int;
    pub fn groove_encoder_detach(encoder: *mut GrooveEncoder) -> c_int;
    pub fn groove_encoder_buffer_get(encoder: *mut GrooveEncoder, buffer: *mut *mut GrooveBuffer,
                                 block: c_int) -> c_int;

    pub fn groove_buffer_unref(buffer: *mut GrooveBuffer);

    pub fn groove_sink_create() -> *mut GrooveSink;
    pub fn groove_sink_destroy(sink: *mut GrooveSink);
    pub fn groove_sink_attach(sink: *mut GrooveSink, playlist: *mut GroovePlaylist) -> c_int;
    pub fn groove_sink_detach(sink: *mut GrooveSink) -> c_int;
    pub fn groove_sink_buffer_get(sink: *mut GrooveSink, buffer: *mut *mut GrooveBuffer,
                              block: c_int) -> c_int;
}