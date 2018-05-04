use std::mem::transmute;
use std::slice;

use c_api::{
    GrooveBuffer,
    BUFFER_NO,
    BUFFER_YES,
    BUFFER_END,
    groove_buffer_unref,
    groove_channel_layout_count,
    groove_encoder_buffer_get,
    groove_sink_buffer_get
};

use audio_format::{SampleFormat, SampleType};
use encoder::Encoder;
use sink::Sink;

/// A buffer which contains encoded audio data
pub struct EncodedBuffer {
    pub(crate) groove_buffer: *mut GrooveBuffer,
}
unsafe impl Sync for EncodedBuffer {}
unsafe impl Send for EncodedBuffer {}

impl Drop for EncodedBuffer {
    fn drop(&mut self) {
        unsafe {
            groove_buffer_unref(self.groove_buffer);
        }
    }
}

impl EncodedBuffer {
    pub(crate) fn from_encoder(encoder: &Encoder) -> Result<Option<EncodedBuffer>, ()> {
        let mut groove_buffer: *mut GrooveBuffer = ::std::ptr::null_mut();
        let return_code = unsafe {
            groove_encoder_buffer_get(encoder.groove_encoder, &mut groove_buffer, 1)
        };

        match return_code {
            c if c == BUFFER_NO => Err(()),
            c if c == BUFFER_YES => Ok(Some(EncodedBuffer { groove_buffer })),
            c if c == BUFFER_END => Ok(None),
            _ => panic!("unexpected buffer result"),
        }
    }

    pub fn as_vec(&self) -> &[u8] {
        unsafe {
            let data = *(*self.groove_buffer).data;
            let len = (*self.groove_buffer).size as usize;
            slice::from_raw_parts(data, len)
        }
    }
}

/// A buffer which contains raw samples
pub struct DecodedBuffer {
    pub(crate) groove_buffer: *mut GrooveBuffer,
}
unsafe impl Sync for DecodedBuffer {}
unsafe impl Send for DecodedBuffer {}

impl Drop for DecodedBuffer {
    fn drop(&mut self) {
        unsafe {
            groove_buffer_unref(self.groove_buffer);
        }
    }
}

impl DecodedBuffer {
    pub(crate) fn from_sink(sink: &Sink) -> Result<Option<DecodedBuffer>, ()> {
        let mut groove_buffer: *mut GrooveBuffer = ::std::ptr::null_mut();
        let return_code = unsafe {
            groove_sink_buffer_get(sink.groove_sink, &mut groove_buffer, 1)
        };

        match return_code {
            c if c == BUFFER_NO => Err(()),
            c if c == BUFFER_YES => Ok(Some(DecodedBuffer { groove_buffer })),
            c if c == BUFFER_END => Ok(None),
            _ => panic!("unexpected buffer result"),
        }
    }

    /// returns a vector of f64
    /// panics if the buffer is not planar
    /// panics if the buffer is not SampleType::Dbl
    pub fn channel_as_slice_f64(&self, channel_index: u32) -> &[f64] {
        match self.sample_format().sample_type {
            SampleType::Dbl => self.channel_as_slice_generic(channel_index),
            _ => panic!("buffer not in f64 format"),
        }
    }

    /// returns a vector of f32
    /// panics if the buffer is not planar
    /// panics if the buffer is not SampleType::Flt
    pub fn channel_as_slice_f32(&self, channel_index: u32) -> &[f32] {
        match self.sample_format().sample_type {
            SampleType::Flt => self.channel_as_slice_generic(channel_index),
            _ => panic!("buffer not in f32 format"),
        }
    }

    /// returns a vector of i32
    /// panics if the buffer is not planar
    /// panics if the buffer is not SampleType::S32
    pub fn channel_as_slice_i32(&self, channel_index: u32) -> &[i32] {
        match self.sample_format().sample_type {
            SampleType::S32 => self.channel_as_slice_generic(channel_index),
            _ => panic!("buffer not in i32 format"),
        }
    }

    /// returns a vector of i16
    /// panics if the buffer is not planar
    /// panics if the buffer is not SampleType::S16
    pub fn channel_as_slice_i16(&self, channel_index: u32) -> &[i16] {
        match self.sample_format().sample_type {
            SampleType::S16 => self.channel_as_slice_generic(channel_index),
            _ => panic!("buffer not in i16 format"),
        }
    }

    /// returns a vector of u8
    /// panics if the buffer is not planar
    /// panics if the buffer is not SampleType::U8
    pub fn channel_as_slice_u8(&self, channel_index: u32) -> &[u8] {
        match self.sample_format().sample_type {
            SampleType::U8 => self.channel_as_slice_generic(channel_index),
            _ => panic!("buffer not in u8 format"),
        }
    }

    pub fn sample_format(&self) -> SampleFormat {
        unsafe {
            SampleFormat::from_groove((*self.groove_buffer).format.sample_fmt)
        }
    }

    fn channel_as_slice_generic<T>(&self, channel_index: u32) -> &[T] {
        unsafe {
            let sample_fmt = self.sample_format();
            if !sample_fmt.planar {
                panic!("expected planar buffer");
            }
            let channel_count = groove_channel_layout_count(
                (*self.groove_buffer).format.channel_layout) as u32;
            if channel_index >= channel_count {
                panic!("invalid channel index");
            }
            let data = *((*self.groove_buffer).data.offset(channel_index as isize));
            let frame_count = (*self.groove_buffer).frame_count as usize;
            let raw_slice = slice::from_raw_parts(data, frame_count);
            transmute(raw_slice)
        }
    }

    /// returns a single channel and always returns [u8]
    /// panics if the buffer is not planar
    pub fn channel_as_slice_raw(&self, channel_index: u32) -> &[u8] {
        self.channel_as_slice_generic(channel_index)
    }

    /// returns a vector of f64
    /// panics if the buffer is planar
    /// panics if the buffer is not SampleType::Dbl
    pub fn as_slice_f64(&self) -> &[f64] {
        match self.sample_format().sample_type {
            SampleType::Dbl => self.as_slice_generic(),
            _ => panic!("buffer not in f64 format"),
        }
    }

    /// returns a vector of f32
    /// panics if the buffer is planar
    /// panics if the buffer is not SampleType::Flt
    pub fn as_slice_f32(&self) -> &[f32] {
        match self.sample_format().sample_type {
            SampleType::Flt => self.as_slice_generic(),
            _ => panic!("buffer not in f32 format"),
        }
    }

    /// returns a vector of i32
    /// panics if the buffer is planar
    /// panics if the buffer is not SampleType::S32
    pub fn as_slice_i32(&self) -> &[i32] {
        match self.sample_format().sample_type {
            SampleType::S32 => self.as_slice_generic(),
            _ => panic!("buffer not in i32 format"),
        }
    }

    /// returns a vector of i16
    /// panics if the buffer is planar
    /// panics if the buffer is not SampleType::S16
    pub fn as_slice_i16(&self) -> &[i16] {
        match self.sample_format().sample_type {
            SampleType::S16 => self.as_slice_generic(),
            _ => panic!("buffer not in i16 format"),
        }
    }

    /// returns a vector of u8
    /// panics if the buffer is planar
    /// panics if the buffer is not SampleType::U8
    pub fn as_slice_u8(&self) -> &[u8] {
        match self.sample_format().sample_type {
            SampleType::U8 => self.as_slice_generic(),
            _ => panic!("buffer not in u8 format"),
        }
    }

    /// returns all the buffer data as [u8]
    /// panics if the buffer is planar
    pub fn as_slice_raw(&self) -> &[u8] {
        self.as_slice_generic()
    }

    fn as_slice_generic<T>(&self) -> &[T] {
        unsafe {
            let sample_fmt = (*self.groove_buffer).format.sample_fmt;
            if SampleFormat::from_groove(sample_fmt).planar {
                panic!("as_vec works for interleaved buffers only");
            }
            let channel_count = groove_channel_layout_count(
                (*self.groove_buffer).format.channel_layout) as usize;
            let frame_count = (*self.groove_buffer).frame_count as usize;
            let data = *(*self.groove_buffer).data;
            let len = channel_count * frame_count;
            let raw_slice = slice::from_raw_parts(data, len);
            transmute(raw_slice)
        }
    }
}