use eframe::egui;

pub struct HexViewer {
    // maybe cache lines?
}

impl HexViewer {
    pub fn show(ui: &mut egui::Ui, data: &[u8]) {
        ui.label(format!("Raw Data ({} bytes)", data.len()));
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.style_mut().spacing.item_spacing.y = 0.0;
            // Use monospace font
            let font_id = egui::FontId::monospace(12.0);
            
            // Chunk by 16 bytes
            for (i, chunk) in data.chunks(16).enumerate() {
                let offset = i * 16;
                let hex_part: String = chunk.iter().map(|b| format!("{:02X} ", b)).collect();
                let ascii_part: String = chunk.iter().map(|&b| {
                    if b >= 32 && b <= 126 { b as char } else { '.' }
                }).collect();
                
                // Pad hex part if chunk is less than 16
                let padded_hex = if chunk.len() < 16 {
                     format!("{:width$}", hex_part, width = 16 * 3)
                } else {
                     hex_part
                };

                let text = format!("{:08X}: {} | {}", offset, padded_hex, ascii_part);
                ui.label(egui::RichText::new(text).font(font_id.clone()));
            }
        });
    }
}
