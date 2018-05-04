use c_api::{
    GrooveSink,
    groove_sink_attach,
    groove_sink_detach,
    groove_sink_create,
    groove_sink_destroy,
};

use audio_format::AudioFormat;
use buffer::DecodedBuffer;
use playlist::Playlist;

/// use this to get access to a realtime raw audio buffer
/// for example you could use it to draw a waveform or other visualization
/// GroovePlayer uses this internally to get the audio buffer for playback
pub struct Sink {
    pub(crate) groove_sink: *mut GrooveSink,
}

impl Drop for Sink {
    fn drop(&mut self) {
        unsafe {
            if !(*self.groove_sink).playlist.is_null() {
                groove_sink_detach(self.groove_sink);
            }
            groove_sink_destroy(self.groove_sink)
        }
    }
}

impl Sink {
    pub fn new() -> Self {
        super::init();
        unsafe {
            Sink { groove_sink: groove_sink_create() }
        }
    }

    /// set this to the audio format you want the sink to output
    pub fn set_audio_format(&self, format: AudioFormat) {
        unsafe {
            (*self.groove_sink).audio_format = format.to_groove();
        }
    }

    pub fn attach(&self, playlist: &Playlist) -> Result<(), i32> {
        unsafe {
            let err_code = groove_sink_attach(self.groove_sink, playlist.groove_playlist);
            if err_code >= 0 {
                Result::Ok(())
            } else {
                Result::Err(err_code as i32)
            }
        }
    }

    pub fn detach(&self) {
        unsafe {
            let _ = groove_sink_detach(self.groove_sink);
        }
    }

    /// returns None on end of playlist, Some<DecodedBuffer> when there is a buffer
    /// blocks the thread until a buffer or end is found
    pub fn buffer_get_blocking(&self) -> Option<DecodedBuffer> {
        DecodedBuffer::from_sink(self).expect("buffer aborted or not ready")

    }

    /// Set this flag to ignore audio_format. If you set this flag, the
    /// buffers you pull from this sink could have any audio format.
    pub fn disable_resample(&self, disabled: bool) {
        unsafe {
            (*self.groove_sink).disable_resample = if disabled { 1 } else { 0 }
        }
    }
}
