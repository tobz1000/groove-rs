extern crate libc;

use std::ffi::CString;

use libc::c_int;

use c_api::{
    GrooveEncoder,
    TAG_MATCH_CASE,
    groove_encoder_attach,
    groove_encoder_detach,
    groove_encoder_create,
    groove_encoder_destroy,
    groove_encoder_metadata_set,
};
use audio_format::AudioFormat;
use buffer::EncodedBuffer;
use playlist::Playlist;

/// attach an Encoder to a playlist to keep a buffer of encoded audio full.
/// for example you could use it to implement an http audio stream
pub struct Encoder {
    pub(crate) groove_encoder: *mut GrooveEncoder,
}

impl Drop for Encoder {
    fn drop(&mut self) {
        unsafe {
            if !(*self.groove_encoder).playlist.is_null() {
                groove_encoder_detach(self.groove_encoder);
            }
            groove_encoder_destroy(self.groove_encoder)
        }
    }
}

impl Encoder {
    pub fn new() -> Self {
        super::init();
        unsafe {
            Encoder { groove_encoder: groove_encoder_create() }
        }
    }

    /// The desired audio format to encode.
    /// groove_encoder_create defaults these to 44100 Hz,
    /// signed 16-bit int, stereo.
    /// These are preferences; if a setting cannot be used, a substitute will be
    /// used instead. actual_audio_format is set to the actual values.
    pub fn set_target_audio_format(&self, target_audio_format: AudioFormat) {
        unsafe {
            (*self.groove_encoder).target_audio_format = target_audio_format.to_groove();
        }
    }
    pub fn get_target_audio_format(&self) -> AudioFormat {
        unsafe {
            AudioFormat::from_groove(&(*self.groove_encoder).target_audio_format)
        }
    }

    /// Select encoding quality by choosing a target bit rate in bits per
    /// second. Note that typically you see this expressed in "kbps", such
    /// as 320kbps or 128kbps. Surprisingly, in this circumstance 1 kbps is
    /// 1000 bps, *not* 1024 bps as you would expect.
    /// groove_encoder_create defaults this to 256000
    pub fn set_bit_rate(&self, rate: i32) {
        unsafe {
            (*self.groove_encoder).bit_rate = rate;
        }
    }
    pub fn get_bit_rate(&self) -> i32 {
        unsafe {
            (*self.groove_encoder).bit_rate
        }
    }

    /// optional - choose a short name for the format
    /// to help libgroove guess which format to use
    /// use `avconv -formats` to get a list of possibilities
    pub fn set_format_short_name(&self, format: &str) {
        let format_c_str = CString::new(format).unwrap();
        unsafe {
            (*self.groove_encoder).format_short_name = format_c_str.as_ptr();
        }
    }

    /// optional - choose a short name for the codec
    /// to help libgroove guess which codec to use
    /// use `avconv -codecs` to get a list of possibilities
    pub fn set_codec_short_name(&self, codec: &str) {
        let codec_c_str = CString::new(codec).unwrap();
        unsafe {
            (*self.groove_encoder).codec_short_name = codec_c_str.as_ptr();
        }
    }

    /// optional - provide an example filename
    /// to help libgroove guess which format/codec to use
    pub fn set_filename(&self, filename: &str) {
        let filename_c_str = CString::new(filename).unwrap();
        unsafe {
            (*self.groove_encoder).filename = filename_c_str.as_ptr();
        }
    }

    /// optional - provide a mime type string
    /// to help libgroove guess which format/codec to use
    pub fn set_mime_type(&self, mime_type: &str) {
        let mime_type_c_str = CString::new(mime_type).unwrap();
        unsafe {
            (*self.groove_encoder).mime_type = mime_type_c_str.as_ptr();
        }
    }

    /// set to the actual format you get when you attach to a
    /// playlist. ideally will be the same as target_audio_format but might
    /// not be.
    pub fn get_actual_audio_format(&self) -> AudioFormat {
        unsafe {
            AudioFormat::from_groove(&(*self.groove_encoder).actual_audio_format)
        }
    }

    /// see docs for file::metadata_set
    pub fn metadata_set(&self, key: &str, value: &str, case_sensitive: bool) -> Result<(), i32> {
        let flags: c_int = if case_sensitive {TAG_MATCH_CASE} else {0};
        let c_tag_key = CString::new(key).unwrap();
        let c_tag_value = CString::new(value).unwrap();
        unsafe {
            let err_code = groove_encoder_metadata_set(self.groove_encoder, c_tag_key.as_ptr(),
                                                       c_tag_value.as_ptr(), flags);
            if err_code >= 0 {
                Result::Ok(())
            } else {
                Result::Err(err_code as i32)
            }
        }
    }

    /// at playlist begin, format headers are generated. when end of playlist is
    /// reached, format trailers are generated.
    pub fn attach(&self, playlist: &Playlist) -> Result<(), i32> {
        unsafe {
            let err_code = groove_encoder_attach(self.groove_encoder, playlist.groove_playlist);
            if err_code >= 0 {
                Result::Ok(())
            } else {
                Result::Err(err_code as i32)
            }
        }
    }

    pub fn detach(&self) {
        unsafe {
            let _ = groove_encoder_detach(self.groove_encoder);
        }
    }

    /// returns None on end of playlist, Some<EncodedBuffer> when there is a buffer
    /// blocks the thread until a buffer or end is found
    pub fn buffer_get_blocking(&self) -> Option<EncodedBuffer> {
        EncodedBuffer::from_encoder(self).expect("buffer aborted or not ready")
    }
}