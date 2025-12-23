use std::io::{self, Cursor, Read};
use byteorder::{ByteOrder, LittleEndian};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleInfo {
    pub name: String,
    pub uncompressed_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path_hash: u64,
    pub bundle_index: u32,
    pub file_offset: u32,
    pub file_size: u32,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryInfo {
    pub path_hash: u64,
    pub offset: u32,
    pub size: u32,
    pub recursive_size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    pub bundles: Vec<BundleInfo>,
    pub files: HashMap<u64, FileInfo>,
}

impl Index {
    pub fn read(data: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(data);
        
        // Read Bundle Count
        let bundle_count = read_i32(&mut cursor)?;
        let mut bundles = Vec::with_capacity(bundle_count as usize);
        
        for _ in 0..bundle_count {
            let name_len = read_i32(&mut cursor)?;
            let mut name_buf = vec![0u8; name_len as usize];
            cursor.read_exact(&mut name_buf)?;
            let name = String::from_utf8_lossy(&name_buf).to_string();
            
            let uncompressed_size = read_u32(&mut cursor)?;
            bundles.push(BundleInfo { name, uncompressed_size });
        }
        
        let file_count = read_i32(&mut cursor)?;
        println!("Index::read: Found {} files", file_count);
        let mut files_map = HashMap::with_capacity(file_count as usize);
        
        for _ in 0..file_count {
            let path_hash = read_u64(&mut cursor)?;
            let bundle_index = read_u32(&mut cursor)?;
            let file_offset = read_u32(&mut cursor)?;
            let file_size = read_u32(&mut cursor)?;
            
            files_map.insert(path_hash, FileInfo { 
                path_hash, 
                bundle_index, 
                file_offset, 
                file_size,
                path: String::new(), // To be filled
            });
        }
        
        let directory_count = read_i32(&mut cursor)?;
        let mut directories = Vec::with_capacity(directory_count as usize);
        
        for _ in 0..directory_count {
            let path_hash = read_u64(&mut cursor)?;
            let offset = read_u32(&mut cursor)?;
            let size = read_u32(&mut cursor)?;
            let recursive_size = read_u32(&mut cursor)?;
            
            directories.push(DirectoryInfo { path_hash, offset, size, recursive_size });
        }
        
        let current_pos = cursor.position() as usize;
        let directory_bundle_data = &data[current_pos..];
        
        // Parse Paths
        let mut dir_cursor = Cursor::new(directory_bundle_data);
        if let Ok(bundle) = crate::bundles::bundle::Bundle::read_header(&mut dir_cursor) {
             if let Ok(dir_data) = bundle.decompress(&mut dir_cursor) {
                 Self::parse_paths(&directories, &dir_data, &mut files_map);
             } else {
                 println!("Failed to decompress directory bundle");
             }
        } else {
            println!("Failed to read directory bundle header");
        }

        let populated_count = files_map.values().filter(|f| !f.path.is_empty()).count();
        println!("Index::read: {}/{} files have paths", populated_count, files_map.len());
        
        Ok(Self { bundles, files: files_map })
    }

    pub fn save_to_cache<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);
        bincode::serialize_into(&mut writer, self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    pub fn load_from_cache<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);
        bincode::deserialize_from(&mut reader)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn parse_paths(directories: &[DirectoryInfo], dir_data: &[u8], files: &mut HashMap<u64, FileInfo>) {
        if dir_data.is_empty() { return; }

        for d in directories {
            if d.offset as usize >= dir_data.len() { continue; }
            
            let start = d.offset as usize;
            let end = start + d.size as usize;
            if end > dir_data.len() { continue; }
            
            let chunk = &dir_data[start..end];
            let mut ptr = 0;
            
            // Path stack for reconstruction
            let _paths: Vec<Vec<u8>> = Vec::new(); // UNUSED
            let _ = _paths;
            let mut is_directory_phase = true; // First phase is directories (base paths), second is files

            // Read loop
            while ptr < chunk.len() {
                if chunk.len() - ptr < 4 { break; }
                let length_val = LittleEndian::read_u32(&chunk[ptr..ptr+4]);
                ptr += 4;

                if length_val == 0 {
                    // Switch phase
                    is_directory_phase = !is_directory_phase;
                    if is_directory_phase {
                        // Reset stack if we switch back to dir phase? 
                        // Actually LibBundle3 clears it on phase switch FROM 0?
                        // "index == 0" -> Toggle Base. If Base -> Clear().
                        // So if we hit 0, we toggle `base`. If `base` becomes true, we clear.
                        // Here `is_directory_phase` acts as `base`.
                        // Initially `base = false` in LibBundle3.
                        // Wait, let's trace carefully:
                        // bool base = false;
                        // while (...) { int index = Read(); if (index == 0) { base = !base; if (base) temp.Clear(); } ... }
                    }
                    // Wait, implementing EXACTLY as LibBundle3 describes logic:
                    // int index = ReadInt();
                    // if (index == 0) { base = !base; if (base) temp.Clear(); continue; }
                    
                    // But here we read length_val? No, it's an index/length combo.
                    // Actually, the format is run-length encoded somewhat.
                    // The value read IS the 'index'.
                    
                    // Re-implementing strictly based on findings:
                    // The "0" marker acts as a delimiter between sections.
                    
                    // Let's retry the exact logic within this block:
                    continue; 
                }
                
                // It was NOT 0, so it's a valid entry.
                // In previous code we treated it as index.
                // Let's follow the previous logic structure but correct the stack handling.
                let _index = length_val as usize; // It's actually index - 1 in logic usually?
    
                 // It seems the loop structure in my previous Replace was better but buggy.
                 // Let's restart the loop logic completely using `continue` approach.
            }
        }
        
        // RE-WRITING THE ENTIRE METHOD CAREFULLY
        // Based on LibGGPK3 / PyPoE / open source references for Bundle Index.
        // It seems to be:
        // [Index] [StringZero]
        // If Index == 0: Toggle 'Base' mode. If Base is now true, Clear TempStack.
        // If Index != 0: 
        //    Offset = Index - 1
        //    Str = ReadString()
        //    NewPath = (Offset < TempStack.Count) ? TempStack[Offset] + Str : Str
        //    If Base: TempStack.Add(NewPath)
        //    Else: Hash(NewPath) -> Add to Files
            
        for d in directories {
            if d.offset as usize >= dir_data.len() { continue; }
            let start = d.offset as usize;
            let end = start + d.size as usize;
            if end > dir_data.len() { continue; }
            
            let chunk = &dir_data[start..end];
            let mut ptr = 0;
            let mut temp: Vec<Vec<u8>> = Vec::new();
            let mut base = false;

            while ptr + 4 <= chunk.len() {
                let val = LittleEndian::read_u32(&chunk[ptr..ptr+4]);
                ptr += 4;

                if val == 0 {
                    base = !base;
                    if base {
                        temp.clear();
                    }
                    continue;
                }

                let idx = (val - 1) as usize;
                
                // Read String
                let mut str_len = 0;
                while ptr + str_len < chunk.len() && chunk[ptr + str_len] != 0 {
                    str_len += 1;
                }
                
                let s_bytes = if ptr + str_len < chunk.len() {
                    &chunk[ptr..ptr+str_len]
                } else {
                    &[]
                };
                
                // Construct full path content
                let full_path_bytes = if idx < temp.len() {
                    let mut p = temp[idx].clone();
                    p.extend_from_slice(s_bytes);
                    p
                } else {
                    s_bytes.to_vec()
                };
                
                ptr += str_len + 1; // +1 for null

                if base {
                    // In 'base' mode, we are building directory prefixes.
                    // We must push to temp.
                    if idx < temp.len() {
                         // If we built upon an existing one, update or push?
                         // LibBundle3 does `temp.Add(path)` always.
                         // But if we used idx to reference it...
                         // Actually LibBundle3 implementation:
                         // var path = (offset < temp.Count) ? temp[offset] + str : str;
                         // if (base) temp.Add(path);
                         temp.push(full_path_bytes);
                    } else {
                         // If idx >= temp.len, it means we are starting a fresh root in the stack?
                         // Or maybe `s_bytes` is the whole thing.
                         temp.push(full_path_bytes);
                    }
                } else {
                    // File Mode
                    // Try to match hashes
                    let lower_bytes = full_path_bytes.to_ascii_lowercase();
                    
                    // Try Murmur64A (PoE 2)
                    let hash_murmur = murmur_hash64a(&full_path_bytes);
                    let hash_murmur_lower = murmur_hash64a(&lower_bytes);
                    
                    // Try FNV (PoE 1/Older)
                    let hash_fnv = fnv1a64(&full_path_bytes);
                    let hash_fnv_lower = fnv1a64(&lower_bytes);

                    // Refactored Helper to assign path
                    // Refactored Helper to assign path
                    if let Some(f) = files.get_mut(&hash_murmur) { f.path = String::from_utf8_lossy(&full_path_bytes).to_string(); }
                    else if let Some(f) = files.get_mut(&hash_murmur_lower) { f.path = String::from_utf8_lossy(&full_path_bytes).to_string(); }
                    else if let Some(f) = files.get_mut(&hash_fnv) { f.path = String::from_utf8_lossy(&full_path_bytes).to_string(); }
                    else if let Some(f) = files.get_mut(&hash_fnv_lower) { f.path = String::from_utf8_lossy(&full_path_bytes).to_string(); }
                }
            }
        }
    }
}

fn murmur_hash64a(key: &[u8]) -> u64 {
    let seed: u64 = 0x1337B33F;
    let m: u64 = 0xc6a4a7935bd1e995;
    let r: i32 = 47;

    let len = key.len();
    let mut h: u64 = seed ^ ((len as u64).wrapping_mul(m));

    let n_blocks = len / 8;
    let mut data = key;

    for _ in 0..n_blocks {
        let mut k = LittleEndian::read_u64(&data[0..8]);

        k = k.wrapping_mul(m);
        k ^= k >> r;
        k = k.wrapping_mul(m);

        h ^= k;
        h = h.wrapping_mul(m);

        data = &data[8..];
    }

    let remaining = &data;
    if !remaining.is_empty() {
        // C++:
        // switch (len & 7) {
        // case 7: h ^= uint64_t(data2[6]) << 48;
        // case 6: h ^= uint64_t(data2[5]) << 40;
        // ...
        // case 1: h ^= uint64_t(data2[0]);
        //         h *= m;
        // };

        let len_rem = len & 7;
        if len_rem >= 7 { h ^= (remaining[6] as u64) << 48; }
        if len_rem >= 6 { h ^= (remaining[5] as u64) << 40; }
        if len_rem >= 5 { h ^= (remaining[4] as u64) << 32; }
        if len_rem >= 4 { h ^= (remaining[3] as u64) << 24; }
        if len_rem >= 3 { h ^= (remaining[2] as u64) << 16; }
        if len_rem >= 2 { h ^= (remaining[1] as u64) << 8; }
        if len_rem >= 1 { 
            h ^= remaining[0] as u64; 
            h = h.wrapping_mul(m);
        }
    }

    h ^= h >> r;
    h = h.wrapping_mul(m);
    h ^= h >> r;

    h
}

fn fnv1a64(key: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in key {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}


fn read_i32<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(LittleEndian::read_i32(&buf))
}

fn read_u32<R: Read>(reader: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(LittleEndian::read_u32(&buf))
}

fn read_u64<R: Read>(reader: &mut R) -> io::Result<u64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(LittleEndian::read_u64(&buf))
}
