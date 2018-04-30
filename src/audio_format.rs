extern crate libc;

use libc::{uint64_t, c_int};

use c_api::{
    GrooveAudioFormat,
    SAMPLE_FMT_NONE,
    SAMPLE_FMT_U8,
    SAMPLE_FMT_S16,
    SAMPLE_FMT_S32,
    SAMPLE_FMT_FLT,
    SAMPLE_FMT_DBL,
    SAMPLE_FMT_U8P,
    SAMPLE_FMT_S16P,
    SAMPLE_FMT_S32P,
    SAMPLE_FMT_FLTP,
    SAMPLE_FMT_DBLP,
    CH_FRONT_CENTER,
    CH_FRONT_LEFT,
    CH_FRONT_RIGHT,
    CH_LAYOUT_MONO,
    CH_LAYOUT_STEREO,
    groove_channel_layout_default,
    groove_channel_layout_count,
    groove_sample_format_bytes_per_sample,
};

#[derive(Clone, Copy, Debug)]
pub enum ChannelLayout {
    FrontLeft,
    FrontRight,
    FrontCenter,
    LayoutMono,
    LayoutStereo,
}

impl ChannelLayout {
    /// get the default channel layout based on the channel count
    pub fn default(count: i32) -> Self {
        let x = unsafe { groove_channel_layout_default(count) };
        ChannelLayout::from_groove(x)
    }

    /// Get the channel count for the channel layout
    pub fn count(&self) -> i32 {
        unsafe { groove_channel_layout_count(self.to_groove()) as i32 }
    }

    fn to_groove(&self) -> uint64_t {
        match *self {
            ChannelLayout::FrontLeft    => CH_FRONT_LEFT,
            ChannelLayout::FrontRight   => CH_FRONT_RIGHT,
            ChannelLayout::FrontCenter  => CH_FRONT_CENTER,
            ChannelLayout::LayoutMono   => CH_LAYOUT_MONO,
            ChannelLayout::LayoutStereo => CH_LAYOUT_STEREO,
        }
    }

    fn from_groove(x: uint64_t) -> Self {
        match x {
            CH_FRONT_LEFT     => ChannelLayout::FrontLeft,
            CH_FRONT_RIGHT    => ChannelLayout::FrontRight,
            CH_FRONT_CENTER   => ChannelLayout::FrontCenter,
            CH_LAYOUT_STEREO  => ChannelLayout::LayoutStereo,
            _                 => panic!("invalid channel layout"),
        }
    }
}

/// how to organize bits which represent audio samples
#[derive(Clone, Copy)]
pub struct SampleFormat {
    pub sample_type: SampleType,
    /// planar means non-interleaved
    pub planar: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum SampleType {
    NoType,
    /// unsigned 8 bits
    U8,
    /// signed 16 bits
    S16,
    /// signed 32 bits
    S32,
    /// float (32 bits)
    Flt,
    /// double (64 bits)
    Dbl,
}

impl SampleFormat {
    fn to_groove(&self) -> i32 {
        match (self.sample_type, self.planar) {
            (SampleType::NoType, false) => SAMPLE_FMT_NONE,
            (SampleType::U8,     false) => SAMPLE_FMT_U8,
            (SampleType::S16,    false) => SAMPLE_FMT_S16,
            (SampleType::S32,    false) => SAMPLE_FMT_S32,
            (SampleType::Flt,    false) => SAMPLE_FMT_FLT,
            (SampleType::Dbl,    false) => SAMPLE_FMT_DBL,

            (SampleType::NoType, true)  => SAMPLE_FMT_NONE,
            (SampleType::U8,     true)  => SAMPLE_FMT_U8P,
            (SampleType::S16,    true)  => SAMPLE_FMT_S16P,
            (SampleType::S32,    true)  => SAMPLE_FMT_S32P,
            (SampleType::Flt,    true)  => SAMPLE_FMT_FLTP,
            (SampleType::Dbl,    true)  => SAMPLE_FMT_DBLP,
        }
    }

    pub(crate) fn from_groove(groove_sample_format: i32) -> SampleFormat {
        match groove_sample_format {
            SAMPLE_FMT_NONE => SampleFormat { sample_type: SampleType::NoType, planar: false },
            SAMPLE_FMT_U8   => SampleFormat { sample_type: SampleType::U8,     planar: false },
            SAMPLE_FMT_S16  => SampleFormat { sample_type: SampleType::S16,    planar: false },
            SAMPLE_FMT_S32  => SampleFormat { sample_type: SampleType::S32,    planar: false },
            SAMPLE_FMT_FLT  => SampleFormat { sample_type: SampleType::Flt,    planar: false },
            SAMPLE_FMT_DBL  => SampleFormat { sample_type: SampleType::Dbl,    planar: false },

            SAMPLE_FMT_U8P  => SampleFormat { sample_type: SampleType::U8,     planar: true },
            SAMPLE_FMT_S16P => SampleFormat { sample_type: SampleType::S16,    planar: true },
            SAMPLE_FMT_S32P => SampleFormat { sample_type: SampleType::S32,    planar: true },
            SAMPLE_FMT_FLTP => SampleFormat { sample_type: SampleType::Flt,    planar: true },
            SAMPLE_FMT_DBLP => SampleFormat { sample_type: SampleType::Dbl,    planar: true },

            _ => panic!("invalid sample format value"),
        }
    }

    pub fn bytes_per_sample(&self) -> u32 {
        unsafe { groove_sample_format_bytes_per_sample(self.to_groove()) as u32 }
    }
}

#[derive(Clone, Copy)]
pub struct AudioFormat {
    pub sample_rate: i32,
    pub channel_layout: ChannelLayout,
    pub sample_fmt: SampleFormat,
}

impl AudioFormat {
    pub(crate) fn from_groove(groove_audio_format: &GrooveAudioFormat) -> Self {
        AudioFormat {
            sample_rate: groove_audio_format.sample_rate as i32,
            channel_layout: ChannelLayout::from_groove(groove_audio_format.channel_layout),
            sample_fmt: SampleFormat::from_groove(groove_audio_format.sample_fmt),
        }
    }
    pub(crate) fn to_groove(&self) -> GrooveAudioFormat {
        GrooveAudioFormat {
            sample_rate: self.sample_rate as c_int,
            channel_layout: self.channel_layout.to_groove(),
            sample_fmt: self.sample_fmt.to_groove(),
        }
    }
}