extern crate libc;

use std::str;
use std::mem::transmute;
use std::path::Path;
use std::os::unix::ffi::OsStrExt;
use std::ffi::{CStr, CString, OsStr};

use libc::{
    c_char,
    c_void,
    c_int,
};

use c_api::{
    GrooveFile,
    GrooveAudioFormat,
    TAG_MATCH_CASE,
    groove_file_open,
    groove_file_close,
    groove_file_duration,
    groove_file_metadata_get,
    groove_file_metadata_set,
    groove_file_save,
    groove_file_audio_format,
    groove_tag_key,
    groove_tag_value
};

use super::GROOVE_FILE_RC;
use audio_format::AudioFormat;

impl super::Destroy for *mut GrooveFile {
    fn destroy(&self) {
        unsafe {
            groove_file_close(*self);
        }
    }
}

pub struct File {
    pub(crate) groove_file: *mut GrooveFile,
}

impl Drop for File {
    fn drop(&mut self) {
        GROOVE_FILE_RC.lock().unwrap().decr(self.groove_file);
    }
}

impl File {
    /// open a file on disk and prepare to stream audio from it
    pub fn open(filename: &Path) -> Option<File> {
        super::init();
        let filename_byte_vec = filename.as_os_str().as_bytes().to_vec();
        let c_filename = CString::new(filename_byte_vec).unwrap();
        unsafe {
            let groove_file = groove_file_open(c_filename.as_ptr());
            match groove_file.is_null() {
                true  => Option::None,
                false => {
                    GROOVE_FILE_RC.lock().unwrap().incr(groove_file);
                    Option::Some(File { groove_file: groove_file })
                }
            }
        }
    }

    pub fn filename(&self) -> &Path {
        unsafe {
            let slice = CStr::from_ptr((*self.groove_file).filename).to_bytes();
            Path::new(&*(slice as *const [u8] as *const OsStr))
        }
    }
    /// whether the file has pending edits
    pub fn is_dirty(&self) -> bool {
        unsafe {
            (*self.groove_file).dirty == 1
        }
    }
    /// main audio stream duration in seconds. note that this relies on a
    /// combination of format headers and heuristics. It can be inaccurate.
    /// The most accurate way to learn the duration of a file is to use
    /// GrooveLoudnessDetector
    pub fn duration(&self) -> f64 {
        unsafe {
            groove_file_duration(self.groove_file)
        }
    }

    pub fn metadata_get(&self, key: &str, case_sensitive: bool) -> Option<Tag> {
        let flags: c_int = if case_sensitive {TAG_MATCH_CASE} else {0};
        let c_tag_key = CString::new(key).unwrap();
        unsafe {
            let tag = groove_file_metadata_get(self.groove_file, c_tag_key.as_ptr(),
                                               ::std::ptr::null(), flags);
            if tag.is_null() {
                Option::None
            } else {
                Option::Some(Tag {groove_tag: tag})
            }
        }
    }

    pub fn metadata_iter(&self) -> MetadataIterator {
        MetadataIterator { file: self, curr: ::std::ptr::null() }
    }

    pub fn metadata_set(&self, key: &str, value: &str, case_sensitive: bool) -> Result<(), i32> {
        let flags: c_int = if case_sensitive {TAG_MATCH_CASE} else {0};
        let c_tag_key = CString::new(key).unwrap();
        let c_tag_value = CString::new(value).unwrap();
        unsafe {
            let err_code = groove_file_metadata_set(self.groove_file, c_tag_key.as_ptr(),
                                                    c_tag_value.as_ptr(), flags);
            if err_code >= 0 {
                Result::Ok(())
            } else {
                Result::Err(err_code as i32)
            }
        }
    }

    pub fn metadata_delete(&self, key: &str, case_sensitive: bool) -> Result<(), i32> {
        let flags: c_int = if case_sensitive {TAG_MATCH_CASE} else {0};
        let c_tag_key = CString::new(key).unwrap();
        unsafe {
            let err_code = groove_file_metadata_set(self.groove_file, c_tag_key.as_ptr(),
                                                    ::std::ptr::null(), flags);
            if err_code >= 0 {
                Result::Ok(())
            } else {
                Result::Err(err_code as i32)
            }
        }
    }

    /// write changes made to metadata to disk.
    pub fn save(&self) -> Result<(), i32> {
        unsafe {
            let err_code = groove_file_save(self.groove_file);
            if err_code >= 0 {
                Result::Ok(())
            } else {
                Result::Err(err_code as i32)
            }
        }
    }

    /// get the audio format of the main audio stream of a file
    pub fn audio_format(&self) -> AudioFormat {
        unsafe {
            let mut result = GrooveAudioFormat {
                sample_rate: 0,
                channel_layout: 0,
                sample_fmt: 0,
            };
            groove_file_audio_format(self.groove_file, &mut result);
            AudioFormat::from_groove(&result)
        }
    }
}

pub struct MetadataIterator<'a> {
    file: &'a File,
    curr: *const c_void,
}

impl<'a> Iterator for MetadataIterator<'a> {
    type Item = Tag;
    fn next(&mut self) -> Option<Tag> {
        let c_tag_key = CString::new("").unwrap();
        unsafe {
            let tag = groove_file_metadata_get(self.file.groove_file, c_tag_key.as_ptr(),
                                               self.curr, 0);
            self.curr = tag;
            if tag.is_null() {
                Option::None
            } else {
                Option::Some(Tag {groove_tag: tag})
            }
        }
    }
}

pub struct Tag {
    groove_tag: *mut c_void,
}

impl<'a> Tag {
    fn get_field(&self, get: unsafe extern "C" fn(*mut c_void) -> *const c_char) -> Result<&'a str, str::Utf8Error> {
        unsafe {
            let field = get(self.groove_tag);
            let slice = CStr::from_ptr(field).to_bytes();
            str::from_utf8(slice).map(|s| transmute(s))
        }
    }
    pub fn key(&self) -> Result<&'a str, str::Utf8Error> {
        self.get_field(groove_tag_key)

    }
    pub fn value(&self) -> Result<&'a str, str::Utf8Error> {
        self.get_field(groove_tag_value)
    }
}