mod app;
pub mod tree_view; // pub needed for actions
mod content_view;
mod dat_viewer;
pub mod hex_viewer;
pub mod settings_window;
pub mod export_window;

fn load_icon() -> eframe::egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon_bytes = include_bytes!("../../assets/icon-256x256.png");
        let image = image::load_from_memory(icon_bytes)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        (image.into_raw(), width, height)
    };
    
    eframe::egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("GGPK Explorer")
            .with_decorations(true)
            .with_icon(load_icon()),
        ..Default::default()
    };
    
    eframe::run_native(
        "GGPK Explorer",
        options,
        Box::new(|cc| Ok(Box::new(app::ExplorerApp::new(cc)))),
    )
}
