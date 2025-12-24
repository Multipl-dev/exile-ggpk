use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureFormat {
    OriginalDds,
    WebP,
    Png,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioFormat {
    Original, // Usually OGG
    Wav,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataFormat {
    Original,
    Json,
    Csv, // Potential future addition
}

#[derive(Clone)]
pub struct ExportSettings {
    pub texture_format: TextureFormat,
    pub audio_format: AudioFormat,
    pub data_format: DataFormat,
    pub recursive: bool,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            texture_format: TextureFormat::OriginalDds,
            audio_format: AudioFormat::Original,
            data_format: DataFormat::Original,
            recursive: true,
        }
    }
}

pub struct ExportWindow {
    open: bool,
    pub settings: ExportSettings,
    pub confirmed: bool,
    target_name: String,
    is_folder: bool,
    pub hashes: Vec<u64>,
}

impl Default for ExportWindow {
    fn default() -> Self {
        Self {
            open: false,
            settings: ExportSettings::default(),
            confirmed: false,
            target_name: String::new(),
            is_folder: false,
            hashes: Vec::new(),
        }
    }
}

impl ExportWindow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_for(&mut self, name: &str, is_folder: bool) {
        self.open = true;
        self.confirmed = false;
        self.target_name = name.to_string();
        self.is_folder = is_folder;
        // Reset settings to default on new open? Or keep persistence?
        // Let's keep persistence for session-based workflow, but maybe reset recursion if it's a file?
        if !is_folder {
            self.settings.recursive = false;
        } else {
            self.settings.recursive = true;
        }
    }
    
    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut open = self.open;
        if !open { return false; }
        
        let mut confirmed_now = false;
        let mut should_close = false;
        
        egui::Window::new("Export Options")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.heading(format!("Exporting: {}", self.target_name));
                ui.separator();
                
                ui.heading("Texture Options");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.settings.texture_format, TextureFormat::OriginalDds, "Original (DDS)");
                    ui.radio_value(&mut self.settings.texture_format, TextureFormat::WebP, "Convert to WebP");
                    ui.radio_value(&mut self.settings.texture_format, TextureFormat::Png, "Convert to PNG");
                });
                
                ui.add_space(8.0);
                
                ui.heading("Audio Options");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.settings.audio_format, AudioFormat::Original, "Original (OGG/WAV)");
                    ui.radio_value(&mut self.settings.audio_format, AudioFormat::Wav, "Convert to WAV");
                });

                ui.add_space(8.0);
                
                ui.heading("Data Options (.dat)");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.settings.data_format, DataFormat::Original, "Original");
                    ui.radio_value(&mut self.settings.data_format, DataFormat::Json, "Convert to JSON");
                });

                if self.is_folder {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.checkbox(&mut self.settings.recursive, "Recursive Export (Include subfolders)");
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Export").clicked() {
                        self.confirmed = true;
                        confirmed_now = true;
                        should_close = true;
                    }
                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }
                });
            });
            
        if should_close {
            open = false;
        }
        self.open = open;
        confirmed_now
    }
}
