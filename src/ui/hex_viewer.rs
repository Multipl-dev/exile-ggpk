use eframe::egui;

pub struct HexViewer {
    // maybe cache lines?
}

impl HexViewer {
    pub fn show(ui: &mut egui::Ui, data: &[u8]) {
        ui.label(format!("Raw Data ({} bytes)", data.len()));
        
        ui.style_mut().spacing.item_spacing.y = 0.0;
        let font_id = egui::FontId::monospace(12.0);
        
        // Calculate how many bytes fit in one row
        // Row format: "00000000: " (10 chars) + N * "XX " (3 chars) + " | " (3 chars) + N * "c" (1 char)
        // Total chars = 10 + 3*N + 3 + 1*N = 13 + 4*N
        
        let char_width = ui.fonts(|f| f.glyph_width(&font_id, '0'));
        let available_width = ui.available_width().max(300.0); // Safety min
        
        // (available / char) = 13 + 4N
        // 4N = (available / char) - 13
        // N = ((available / char) - 13) / 4
        
        let max_chars = available_width / char_width;
        let mut n = if max_chars > 13.0 {
            ((max_chars - 13.0) / 4.0) as usize
        } else {
            16
        };
        
        // Clamp to multiple of 8 or 16 for readability
        // e.g., if n is 27, snap to 24 or 16? 
        // Let's snap to closest lower multiple of 8
        n = (n / 8) * 8;
        if n < 16 { n = 16; } // Min 16 bytes
        
        // Chunk by N bytes
        for (i, chunk) in data.chunks(n).enumerate() {
            let offset = i * n;
            let hex_part: String = chunk.iter().map(|b| format!("{:02X} ", b)).collect();
            let ascii_part: String = chunk.iter().map(|&b| {
                if b >= 32 && b <= 126 { b as char } else { '.' }
            }).collect();
            
            // Pad hex part if chunk is less than N
            let padded_hex = if chunk.len() < n {
                    format!("{:width$}", hex_part, width = n * 3)
            } else {
                    hex_part
            };

            let text = format!("{:08X}: {} | {}", offset, padded_hex, ascii_part);
            ui.label(egui::RichText::new(text).font(font_id.clone()));
        }
    }
}
