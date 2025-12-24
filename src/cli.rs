use crate::ggpk::reader::GgpkReader;
use crate::settings::AppSettings;

fn murmur_hash64a(key: &[u8], seed: u64) -> u64 {
    let m: u64 = 0xc6a4a7935bd1e995;
    let r: u8 = 47;
    let len = key.len() as u64;
    let mut h: u64 = seed ^ (len.wrapping_mul(m));
    let n_blocks = len / 8;
    let md = key;
    for i in 0..n_blocks {
        let idx = (i * 8) as usize;
        let mut k: u64 = u64::from_le_bytes(md[idx..idx+8].try_into().unwrap());
        k = k.wrapping_mul(m);
        k ^= k >> r;
        k = k.wrapping_mul(m);
        h ^= k;
        h = h.wrapping_mul(m);
    }
    let remainder_idx = (n_blocks * 8) as usize;
    let remaining_len = (len & 7) as usize;
    if remaining_len > 0 {
        let mut k: u64 = 0;
        for i in 0..remaining_len {
             k ^= (md[remainder_idx + i] as u64) << (8 * i);
        }
        h ^= k;
        h = h.wrapping_mul(m);
    }
    h ^= h >> r;
    h = h.wrapping_mul(m);
    h ^= h >> r;
    h
}

pub fn run_inspect() -> Result<(), Box<dyn std::error::Error>> {
    let settings = AppSettings::load();
    let ggpk_path = settings.ggpk_path.ok_or("No GGPK Path")?;
    
    println!("Opening GGPK at: {}", ggpk_path);
    let reader = GgpkReader::open(&ggpk_path)?;
    
    println!("--- GGPK INSPECTOR ---");
    
    // Validate Index
    if let Ok(Some(index_file_record)) = reader.read_file_by_path("Bundles2/_.index.bin") {
        println!("Found Bundles2/_.index.bin");
        let data = reader.get_data_slice(index_file_record.data_offset, index_file_record.data_length)?;
        let mut cursor = std::io::Cursor::new(data);
        
        if let Ok(bundle) = crate::bundles::bundle::Bundle::read_header(&mut cursor) {
             if let Ok(decomp) = bundle.decompress(&mut cursor) {
                 if let Ok(index) = crate::bundles::index::Index::read(&decomp) {
                     println!("Index Loaded: {} files", index.files.len());
                     
                     let target = "data/balance/activeskills.datc64";
                     let hash = murmur_hash64a(target.to_lowercase().as_bytes(), 0x1337b33f);
                     if let Some(file) = index.files.get(&hash) {
                         println!("Verified Hash for '{}': {:016x}", target, hash);
                         println!("  Bundle Index: {}", file.bundle_index);
                     }
                 }
             }
        }
    }
    
    // List Top-Level Dir for context
    if let Ok(entries) = reader.list_files_in_directory("Bundles2") {
        println!("Bundles2 Children: {:?}", entries);
    }
    
    Ok(())
}
