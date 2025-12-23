#![allow(dead_code)]
use byteorder::{ByteOrder, LittleEndian};
use std::io::{self, Cursor, Read};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordTag {
    GGPK,
    PDIR,
    FILE,
    FREE,
    Unknown(u32),
}

impl RecordTag {
    pub const TAG_GGPK: u32 = 0x4B504747;
    pub const TAG_PDIR: u32 = 0x52494450;
    pub const TAG_FILE: u32 = 0x454C4946;
    pub const TAG_FREE: u32 = 0x45455246;

    pub fn from_u32(val: u32) -> Self {
        match val {
            Self::TAG_GGPK => RecordTag::GGPK,
            Self::TAG_PDIR => RecordTag::PDIR,
            Self::TAG_FILE => RecordTag::FILE,
            Self::TAG_FREE => RecordTag::FREE,
            v => RecordTag::Unknown(v),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RecordHeader {
    pub length: u32,
    pub tag: RecordTag,
}

impl RecordHeader {
    pub const SIZE: usize = 8;
    
    pub fn read(data: &[u8]) -> Self {
        let length = LittleEndian::read_u32(&data[0..4]);
        let tag_raw = LittleEndian::read_u32(&data[4..8]);
        let tag = RecordTag::from_u32(tag_raw);
        Self { length, tag }
    }
}

#[derive(Debug)]
pub struct GgpkRecord {
    pub length: u32,
    pub version: u32, 
    pub root_offset: u64,
    pub free_offset: u64,
}

impl GgpkRecord {
    pub fn read(data: &[u8], _offset: u64) -> io::Result<Self> {
        // data starts at the record offset
        // Structure: Length(4), Tag(4), Version(4), RootOffset(8), FreeOffset(8)
        let mut cursor = Cursor::new(data);
        cursor.set_position(8); // header
        
        let version = read_u32(&mut cursor)?;
        let root_offset = read_u64(&mut cursor)?;
        let free_offset = read_u64(&mut cursor)?;
        
        let length = LittleEndian::read_u32(&data[0..4]);

        Ok(Self { length, version, root_offset, free_offset })
    }
}

#[derive(Debug)]
pub struct DirectoryRecord {
    pub length: u32,
    pub offset: u64,
    pub name: String,
    pub hash: [u8; 32],
    pub entries: Vec<DirectoryEntry>,
}

#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name_hash: u32,
    pub offset: u64,
}

impl DirectoryRecord {
    pub fn read(data: &[u8], offset: u64, version: u32) -> io::Result<Self> {
        let mut cursor = Cursor::new(data);
        cursor.set_position(8); // header

        let name_len = read_u32(&mut cursor)?;
        let total_entries = read_u32(&mut cursor)?;
        
        let mut hash = [0u8; 32];
        cursor.read_exact(&mut hash)?;
        
        let name = if version == 4 {
            // UTF-32
            // name_len includes null terminator? LibGGPK3 says read<int>() - 1.
            let actual_len = name_len.saturating_sub(1);
            let mut s = String::with_capacity(actual_len as usize);
            for _ in 0..actual_len {
                let c_val = read_u32(&mut cursor)?;
                if let Some(c) = std::char::from_u32(c_val) {
                    s.push(c);
                } else {
                    s.push('?');
                }
            }
            // Read 4 byte null terminator
            let mut null = [0u8; 4];
            cursor.read_exact(&mut null)?;
            s
        } else {
            // UTF-16
            let actual_name_len = name_len.saturating_sub(1);
            let name_bytes_len = (actual_name_len * 2) as usize;
            
            let mut name_buf = vec![0u8; name_bytes_len];
            cursor.read_exact(&mut name_buf)?;
            
            let mut null_term = [0u8; 2];
            cursor.read_exact(&mut null_term)?;
            
            let name = String::from_utf16_lossy(
                &name_buf.chunks_exact(2).map(|c| LittleEndian::read_u16(c)).collect::<Vec<_>>()
            );
            name
        };

        let mut entries = Vec::with_capacity(total_entries as usize);
        for _ in 0..total_entries {
            let name_hash = read_u32(&mut cursor)?;
            let entry_offset = read_u64(&mut cursor)?;
            entries.push(DirectoryEntry { name_hash, offset: entry_offset });
        }
        
        let length = LittleEndian::read_u32(&data[0..4]);
        
        Ok(Self {
            length,
            offset,
            name,
            hash,
            entries,
        })
    }
}

#[derive(Debug)]
pub struct FileRecord {
    pub length: u32,
    pub offset: u64,
    pub name: String,
    pub hash: [u8; 32],
    pub data_offset: u64,
    pub data_length: u64,
}

impl FileRecord {
    pub fn read(data: &[u8], offset: u64, version: u32) -> io::Result<Self> {
        let mut cursor = Cursor::new(data);
        cursor.set_position(8); // header

        let name_len = read_u32(&mut cursor)?;
        
        let mut hash = [0u8; 32];
        cursor.read_exact(&mut hash)?;
        
        let name = if version == 4 {
            let actual_len = name_len.saturating_sub(1);
            let mut s = String::with_capacity(actual_len as usize);
            for _ in 0..actual_len {
                let c_val = read_u32(&mut cursor)?;
                if let Some(c) = std::char::from_u32(c_val) {
                    s.push(c);
                } else {
                    s.push('?');
                }
            }
            let mut null = [0u8; 4];
            cursor.read_exact(&mut null)?;
            s
        } else {
            let actual_name_len = name_len.saturating_sub(1);
            let name_bytes_len = (actual_name_len * 2) as usize;
            
            let mut name_buf = vec![0u8; name_bytes_len];
            cursor.read_exact(&mut name_buf)?;
            
            let mut null_term = [0u8; 2];
            cursor.read_exact(&mut null_term)?;
            
            let name = String::from_utf16_lossy(
                &name_buf.chunks_exact(2).map(|c| LittleEndian::read_u16(c)).collect::<Vec<_>>()
            );
            name
        };
        
        let header_end = cursor.position();
        let length = LittleEndian::read_u32(&data[0..4]);
        
        // Data length in bytes = Total Length - (HeaderSize + Hash + Name + NullTerm)
        // Which is exactly what cursor consumed (except initial 8 bytes which we skipped but cursor position counts from 0 if we set it)
        // Wait, cursor was created from `data` which is the slice of Record Length.
        // We set position 8.
        // So `header_end` is the index of start of data.
        
        let data_offset = offset + header_end;
        let data_length = length as u64 - header_end;

        Ok(Self {
            length,
            offset,
            name,
            hash,
            data_offset,
            data_length,
        })
    }
}

pub fn read_u32<R: Read>(r: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(LittleEndian::read_u32(&buf))
}

pub fn read_u64<R: Read>(r: &mut R) -> io::Result<u64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(LittleEndian::read_u64(&buf))
}
