#![allow(dead_code)]
use libc::{c_char, c_void, size_t};

pub type BunMem = *mut u8;

#[repr(C)]
pub struct Bun {
    _private: [u8; 0],
}

extern "C" {
    pub fn BunMemAlloc(size: size_t) -> BunMem;
    pub fn BunMemSize(mem: BunMem) -> i64;
    pub fn BunMemFree(mem: BunMem);

    pub fn BunNew(decompressor_path: *const c_char, decompressor_export: *const c_char) -> *mut Bun;
    pub fn BunDelete(bun: *mut Bun);

    // BunMem BunDecompressBundleAlloc(Bun* bun, uint8_t const* src_data, size_t src_size);
    pub fn BunDecompressBundleAlloc(bun: *mut Bun, src_data: *const u8, src_size: size_t) -> BunMem;

    // Direct binding to Ooz_Decompress (wrapper around Kraken_Decompress)
    pub fn Ooz_Decompress(
        src_buf: *const u8, 
        src_len: i32, 
        dst: *mut u8, 
        dst_size: size_t,
        fuzz: i32, 
        crc: i32, 
        verbose: i32, 
        dst_base: *mut u8, 
        e: size_t, 
        cb: *mut c_void, 
        cb_ctx: *mut c_void, 
        scratch: *mut c_void, 
        scratch_size: size_t, 
        threadPhase: i32
    ) -> i32;
}
