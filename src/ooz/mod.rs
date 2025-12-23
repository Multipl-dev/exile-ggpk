#![allow(dead_code)]
pub mod sys;

use std::ffi::CString;
use std::ptr;
// use crate::ooz::sys;

pub struct Bun {
    inner: *mut sys::Bun,
}

impl Bun {
    pub fn new(decompressor_path: &str, decompressor_export: &str) -> Result<Self, String> {
        let c_path = CString::new(decompressor_path).map_err(|e| e.to_string())?;
        let c_export = CString::new(decompressor_export).map_err(|e| e.to_string())?;
        
        let inner = unsafe { sys::BunNew(c_path.as_ptr(), c_export.as_ptr()) };
        if inner.is_null() {
            return Err("Failed to create Bun instance".to_string());
        }
        Ok(Self { inner })
    }

    /// Decompresses a bundle. Returns the raw decompressed bytes.
    pub fn decompress_bundle(&self, src: &[u8]) -> Result<Vec<u8>, String> {
        unsafe {
            let mem = sys::BunDecompressBundleAlloc(self.inner, src.as_ptr(), src.len());
            if mem.is_null() {
                return Err("Failed to decompress bundle".to_string());
            }
            let size = sys::BunMemSize(mem);
            let mut vec = vec![0u8; size as usize];
            ptr::copy_nonoverlapping(mem, vec.as_mut_ptr(), size as usize);
            sys::BunMemFree(mem);
            Ok(vec)
        }
    }
}

impl Drop for Bun {
    fn drop(&mut self) {
        unsafe { sys::BunDelete(self.inner) };
    }
}
