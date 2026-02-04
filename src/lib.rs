//! exile-ggpk - Rust library for reading Path of Exile GGPK and Bundle files
//!
//! Forked from ggpk-explorer by JuddIsJudd (GPL-3.0)
//!
//! # Modules
//!
//! - [`ggpk`] - Classic GGPK format reader (legacy, pre-3.11.2)
//! - [`bundles`] - Bundle format reader (3.11.2+, Oodle compressed)
//! - [`dat`] - Game data file parsing (.dat/.dat64)
//!
//! # Example
//!
//! ```no_run
//! use exile_ggpk::ggpk::reader::GgpkReader;
//!
//! let reader = GgpkReader::open("Content.ggpk").unwrap();
//! let file = reader.read_file_by_path("Data/Items.dat").unwrap();
//! ```

pub mod ggpk;
pub mod bundles;
pub mod dat;
pub mod ooz;

// Re-export commonly used types at crate root
pub use ggpk::reader::GgpkReader;
pub use bundles::index::Index as BundleIndex;
pub use bundles::bundle::Bundle;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ooz_link() {
        // Test that the ooz native library links correctly
        unsafe {
            let ptr = ooz::sys::BunMemAlloc(10);
            assert!(!ptr.is_null());
            ooz::sys::BunMemFree(ptr);
        }
    }
}
