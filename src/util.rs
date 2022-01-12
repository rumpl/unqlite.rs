use crate::error::{Result, Wrap};
use crate::ffi::{
    unqlite_util_load_mmaped_file, unqlite_util_random_num, unqlite_util_random_string,
    unqlite_util_release_mmaped_file,
};
use crate::UnQLite;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;

/// Utility interfaces.
pub trait Util {
    /// Generate random string using the UnQLite PRNG.
    ///
    /// It will generate a english alphabet based string of length buf_size (last argument).
    fn random_string(&self, buf_size: u32) -> Vec<u8>;

    /// Generate random number using the UnQLite PRNG.
    ///
    /// It will return a 32-bit unsigned integer between 0 and 0xFFFFFFFF.
    fn random_num(&self) -> u32;
}

impl Util for UnQLite {
    fn random_string(&self, buf_size: u32) -> Vec<u8> {
        unsafe {
            let vec: Vec<u8> = Vec::with_capacity(buf_size as usize);
            let z_buf = CString::new(vec).unwrap().into_raw();
            unqlite_util_random_string(self.as_raw_mut_ptr(), z_buf, buf_size)
                .wrap()
                .unwrap();
            Vec::from_raw_parts(z_buf as *mut u8, buf_size as usize, buf_size as usize)
        }
    }

    fn random_num(&self) -> u32 {
        unsafe { unqlite_util_random_num(self.as_raw_mut_ptr()) }
    }
}

/// Load memory-mapped file so that we can save it to UnQLite
///
/// NOTE: DONOT USE: will throw unimplemented error.
pub fn load_mmaped_file<P: AsRef<Path>>(path: P) -> Result<Mmap> {
    unsafe {
        let path = path.as_ref();
        let mut ptr: *mut c_void = mem::MaybeUninit::uninit().assume_init();
        let mut size: i64 = 0;
        let cpath = CString::new(path.to_str().expect("cannot convert the path to str"))?;
        unqlite_util_load_mmaped_file(cpath.as_ptr(), &mut ptr, &mut size)
            .wrap()
            .map(|_| Mmap {
                ptr: ptr,
                size: size,
            })
    }
}

/// UnQLite hosted memory mapped file
pub struct Mmap {
    pub ptr: *mut c_void,
    pub size: i64,
}

impl Drop for Mmap {
    fn drop(&mut self) {
        let _ = wrap!(util_release_mmaped_file, self.ptr, self.size);
    }
}

#[cfg(test)]
#[cfg(feature = "enable-threads")]
mod tests {
    use super::*;
    use UnQLite;

    #[test]
    fn test_random_string() {
        let unqlite = UnQLite::create_in_memory();
        let _ = unqlite.random_string(32);
    }

    #[test]
    fn test_random_num() {
        let _ = UnQLite::create_in_memory().random_num();
    }

    #[test]
    #[cfg(feature = "mmap")]
    fn test_mmap() {
        use std::io::Write;
        use tempfile::NamedTempFile;
        let mut f = NamedTempFile::new().expect("get named temp file");
        let _ = f.write_all(b"Hello, world!");
        let _ = f.sync_all();
        load_mmaped_file(f.path()).unwrap();
    }
}
