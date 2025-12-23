use byteorder::{ByteOrder, LittleEndian};
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use super::schema::{Table, Column};

pub struct DatReader {
    data: Vec<u8>,
    pub is_64bit: bool,
    pub row_count: u32,
    pub row_length: Option<usize>, // If fixed length
    pub data_section_offset: u64,
    pub filename: String,
}

impl DatReader {
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn new(data: Vec<u8>, filename: &str) -> io::Result<Self> {
        // Use slice for initial read
        let mut cursor = Cursor::new(data.as_slice());
        
        let is_64bit = filename.ends_with(".dat64") || filename.ends_with(".datc64");

        // DAT format detection (very basic)
        let row_count = read_u32(&mut cursor)?;
        println!("DatReader: Loading {}, Row Count: {}, Is 64bit: {}", filename, row_count, is_64bit);
        
        let mut row_length = None;
        let mut data_section_offset = 0;
        
        // Heuristic: Find 0xBBBBBBBB pattern
        // In 64-bit, it might be 0xBBBBBBBBBBBBBBBB
        if row_count > 0 {
             let pattern_32 = [0xBB, 0xBB, 0xBB, 0xBB];
             let pattern_64 = [0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB];
             
             // Simple scan
             // max_search optimization unused
             
             let mut found_pattern = false;

             for i in 4..data.len().saturating_sub(4) {
                 if is_64bit {
                      if i + 8 <= data.len() && data[i..i+8] == pattern_64 {
                           let data_size = i - 4;
                           println!("DatReader: Found 64-bit pattern at {}, data_size={}, row_count={}", i, data_size, row_count);
                           if data_size % (row_count as usize) == 0 {
                               row_length = Some(data_size / (row_count as usize));
                               data_section_offset = i as u64;
                               found_pattern = true;
                               break;
                           }
                      }
                 } else {
                      if data[i..i+4] == pattern_32 {
                           let data_size = i - 4;
                           println!("DatReader: Found 32-bit pattern at {}, data_size={}, row_count={}", i, data_size, row_count);
                           if data_size % (row_count as usize) == 0 {
                               row_length = Some(data_size / (row_count as usize));
                               data_section_offset = i as u64;
                               found_pattern = true;
                               break;
                           }
                      }
                 }
             }

             if !found_pattern {
                 return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Aligned data boundary not found for row_count {}", row_count)));
             }

        } else {
            println!("DatReader: Row count is 0 for {}", filename);
            // Handle 0 rows?
            row_length = Some(0);
            // Scan for pattern anyway to find data section?
            // If 0 rows, fixed section size is 0?
            // Then pattern should be at offset 4?
             let pattern_32 = [0xBB, 0xBB, 0xBB, 0xBB];
             if data.len() >= 8 && data[4..8] == pattern_32 {
                 data_section_offset = 4;
             }
             // For 64-bit?
             let pattern_64 = [0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB];
             if is_64bit && data.len() >= 12 && data[4..12] == pattern_64 {
                 data_section_offset = 4;
             }
        }
        
        Ok(Self {
            data,
            is_64bit, 
            row_count,
            row_length,
            data_section_offset, 
            filename: filename.to_string(),
        })
    }

    pub fn read_row(&self, index: u32, table: &Table) -> io::Result<Vec<DatValue>> {
        // Graceful handling logic:
        // 1. Calculate expected schema length.
        // 2. Read what we can.
        // 3. If we hit EOF unexpectedly, return what we have or an error value.
        
        let schema_row_len: usize = table.columns.iter().map(|c| get_column_size(c, self.is_64bit)).sum();
        let row_len = self.row_length.unwrap_or(schema_row_len);

        let start = 4 + (index as usize * row_len); // 4 bytes for row count
        if start >= self.data.len() {
             return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Row index out of bounds"));
        }
        
        // Ensure we don't read past EOF even for a valid index if file is truncated
        let end = (start + row_len).min(self.data.len());
        let mut cursor = Cursor::new(&self.data[start..end]);
        
        let mut values = Vec::new();
        
        for col in &table.columns {
             // If we don't have enough bytes for this column, push Error/Unknown
             let needed = get_column_size(col, self.is_64bit);
             let current_pos = cursor.position() as usize;
             if current_pos + needed > cursor.get_ref().len() {
                 values.push(DatValue::Unknown); // Or Error
                 continue;
             }
             
             // Pass separate slice to helper
             match read_column_value(&mut cursor, col, &self.data, self.data_section_offset, self.is_64bit) {
                 Ok(val) => values.push(val),
                 Err(_) => values.push(DatValue::Unknown),
             }
        }
        
        Ok(values)
    }
}

fn get_column_size(col: &Column, is_64bit: bool) -> usize {
    if is_64bit {
        match col.r#type.as_str() {
            "bool" => 1,
            "byte" | "u8" => 1,
            "short" | "u16" => 2,
            "int" | "i32" | "u32" => 4,
            "float" | "f32" => 4,
            "long" | "u64" => 8,
            "string" | "ref|string" => 8, // Offset (8 bytes)
            "ref|list" => 16, // Count (8) + Offset (8)
            "foreign_row" => 16, // Index (8) + Unknown (8) - based on KEY_FOREIGN
            _ => 4,
        }
    } else {
        match col.r#type.as_str() {
            "bool" => 1,
            "byte" | "u8" => 1,
            "short" | "u16" => 2,
            "int" | "i32" | "u32" | "foreign_row" => 4,
            "float" | "f32" => 4,
            "long" | "u64" => 8,
            "string" | "ref|string" => 4, // 32-bit DAT -> 4 bytes offset
            "ref|list" => 8, // Usually Count (4) + Offset (4) in 32-bit
             _ => 4,
        }
    }
}

fn read_column_value(cursor: &mut Cursor<&[u8]>, col: &Column, file_data: &[u8], var_data_offset: u64, is_64bit: bool) -> io::Result<DatValue> {
    match col.r#type.as_str() {
         "bool" => {
             let mut b = [0u8; 1];
             cursor.read_exact(&mut b)?;
             Ok(DatValue::Bool(b[0] != 0))
         },
         "int" | "i32" | "u32" => {
             Ok(DatValue::Int(read_u32(cursor)? as i32))
         },
         "float" | "f32" => {
             let val = read_u32(cursor)?;
             Ok(DatValue::Float(f32::from_bits(val)))
         },
         "long" | "u64" => {
             Ok(DatValue::Long(read_u64(cursor)?))
         },
         "string" | "ref|string" => {
              let offset_val = if is_64bit {
                  read_u64(cursor)?
              } else {
                  read_u32(cursor)? as u64
              };
              
              // In 64-bit DAT, null is 0xfefefefe? or just 0?
              // poe-dat-viewer uses MEM32_NULL for keys.
              // For strings, it reads offset.
              
              let abs_offset = var_data_offset + offset_val;
              
              if (abs_offset as usize) < file_data.len() {
                   // Read UTF-16 string
                   // Search for 0x00000000 (4 bytes) aligned??
                   // poe-dat-viewer: findZeroSequence(data, 4, offset)
                   // and (end - offset) % 2 == 0.
                   
                   let mut end = abs_offset as usize;
                   loop {
                       // Find next 0 byte
                       if let Some(pos) = file_data[end..].iter().position(|&b| b == 0) {
                           end += pos;
                       } else {
                           end = file_data.len();
                           break;
                       }
                       
                       // Check if we have 4 zeros
                       if end + 4 <= file_data.len() && file_data[end..end+4] == [0, 0, 0, 0] {
                           // Check alignment
                           if (end - (abs_offset as usize)) % 2 == 0 {
                               break; // Found it
                           }
                           end += 1; // Not aligned, continue
                       } else {
                           end += 1; // Not 4 zeros, continue
                       }
                       
                       if end >= file_data.len() { break; }
                   }
                   
                   // Decode utf16
                   let byte_slice = &file_data[abs_offset as usize..end];
                   let u16_vec: Vec<u16> = byte_slice
                       .chunks_exact(2)
                       .map(|chunk| LittleEndian::read_u16(chunk))
                       .collect();
                   
                   let s = String::from_utf16_lossy(&u16_vec);
                   Ok(DatValue::String(s))
              } else {
                   Ok(DatValue::String("".to_string()))
              }
         },
         "foreign_row" => {
              let idx = if is_64bit {
                  let v = read_u64(cursor)?;
                  // 16 bytes total, skip next 8
                  let _ = read_u64(cursor)?; 
                  v // Return first 8 bytes as index/key
              } else {
                  read_u32(cursor)? as u64
              };
              
              if idx == 0xfefefefe || idx == 0xfefefefefefefefe {
                  // Null
                  Ok(DatValue::ForeignRow(u32::MAX)) // Sentinel
              } else {
                  Ok(DatValue::ForeignRow(idx as u32))
              }
         },
         "ref|list" => {
             // 16 bytes (Count, Offset)
             if is_64bit {
                 let count = read_u64(cursor)?;
                 let offset = read_u64(cursor)?;
                 Ok(DatValue::List(count as usize, offset))
             } else {
                 let count = read_u32(cursor)?;
                 let offset = read_u32(cursor)?;
                 Ok(DatValue::List(count as usize, offset as u64))
             }
         },
          _ => {
              // Consume bytes
              let size = get_column_size(col, is_64bit);
              if size > 0 { cursor.seek(SeekFrom::Current(size as i64))?; }
              Ok(DatValue::Unknown)
          }
    }
}

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum DatValue {
    Bool(bool),
    Int(i32),
    Long(u64),
    Float(f32),
    String(String),
    ForeignRow(u32),
    List(usize, u64), // Count, Offset
    Unknown,
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf)?;
    Ok(LittleEndian::read_u32(&buf))
}

fn read_u64(cursor: &mut Cursor<&[u8]>) -> io::Result<u64> {
    let mut buf = [0u8; 8];
    cursor.read_exact(&mut buf)?;
    Ok(LittleEndian::read_u64(&buf))
}

