use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;
use reqwest::blocking::Client;
use std::error::Error;

#[derive(Clone)]
pub struct CdnBundleLoader {
    cache_dir: PathBuf,
    client: Client,
    patch_ver: String,
}

impl CdnBundleLoader {
    pub fn new(cache_root: &Path, patch_ver: Option<&str>) -> Self {
        let cache_dir = cache_root.join("Bundles2");
        if !cache_dir.exists() {
            let _ = fs::create_dir_all(&cache_dir);
        }
        CdnBundleLoader {
            cache_dir,
            client: Client::new(),
            patch_ver: patch_ver.unwrap_or("4.4.0.3.7").to_string(), // Default patch version
        }
    }

    pub fn fetch_bundle(&self, bundle_name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        // 1. Check Local Cache
        // Flatten path components for safe caching? 
        // poe-dat-viewer replaces '/' with '@'.
        // Let's mirror that to be safe on Windows.
        let safe_name = bundle_name.replace("/", "@");
        let cache_path = self.cache_dir.join(&safe_name);

        if cache_path.exists() {
            // println!("Loading from cache: {:?}", cache_path);
            let data = fs::read(&cache_path)?;
            return Ok(data);
        }

        // 2. Download from CDN
        let url = if self.patch_ver.starts_with("4.") {
             format!("https://patch-poe2.poecdn.com/{}/Bundles2/{}", self.patch_ver, bundle_name)
        } else {
             format!("https://patch.poecdn.com/{}/Bundles2/{}", self.patch_ver, bundle_name)
        };

        println!("[CDN] Downloading: {}", url);
        let resp = self.client.get(&url).send()?;

        if !resp.status().is_success() {
             return Err(format!("CDN Request Failed: {} ({})", url, resp.status()).into());
        }

        let bytes = resp.bytes()?;
        let data = bytes.to_vec();

        // 3. Save to Cache
        let mut f = fs::File::create(&cache_path)?;
        f.write_all(&data)?;
        
        Ok(data)
    }
    pub fn set_patch_version(&mut self, ver: &str) {
        self.patch_ver = ver.to_string();
    }
}


