extern crate libc;

mod c_api;

mod audio_format;
mod buffer;
mod encoder;
mod file;
mod playlist;
mod sink;

use std::sync::{Once, ONCE_INIT};
use std::ffi::CStr;

use libc::c_int;

use c_api::{
    groove_init,
    groove_finish,
    groove_set_logging,
    groove_version_major,
    groove_version_minor,
    groove_version_patch,
    groove_version,
};
pub use audio_format::{
    ChannelLayout,
    SampleFormat,
    SampleType,
    AudioFormat
};
pub use buffer::{
    EncodedBuffer,
    DecodedBuffer
};
pub use encoder::Encoder;
pub use file::{
    File,
    MetadataIterator,
    Tag
};
pub use playlist::{
    Playlist,
    PlaylistItem,
    FillMode
};
pub use sink::Sink;

fn init() {
    static mut INIT: Once = ONCE_INIT;

    unsafe {
        INIT.call_once(|| {
            let err_code = groove_init();
            if err_code != 0 {
                panic!("groove_init() failed");
            }
        });
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Log {
    Quiet,
    Error,
    Warning,
    Info,
}

/// Call at the end of your program to clean up. After calling this you may no
/// longer use this API. You may choose to never call this function, in which
/// case the worst thing that can happen is valgrind may report a memory leak.
pub fn finish() {
    init();
    unsafe { groove_finish() }
}

/// enable/disable logging of errors
pub fn set_logging(level: Log) {
    init();
    let c_level: c_int = match level {
        Log::Quiet   => -8,
        Log::Error   => 16,
        Log::Warning => 24,
        Log::Info    => 32,
    };
    unsafe { groove_set_logging(c_level) }
}

pub fn version_major() -> i32 {
    unsafe { groove_version_major() }
}

pub fn version_minor() -> i32 {
    unsafe { groove_version_minor() }
}

pub fn version_patch() -> i32 {
    unsafe { groove_version_patch() }
}

/// get a string which represents the version number of libgroove
pub fn version() -> &'static str {
    unsafe {
        let version = groove_version();
        let slice = CStr::from_ptr(version).to_bytes();
        std::str::from_utf8(slice).unwrap()
    }
}