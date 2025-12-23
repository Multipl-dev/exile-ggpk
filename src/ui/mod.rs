mod app;
pub mod tree_view; // pub needed for actions
mod content_view;
mod dat_viewer;
pub mod hex_viewer;

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("GGPK Explorer")
            .with_decorations(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "GGPK Explorer",
        options,
        Box::new(|cc| Ok(Box::new(app::ExplorerApp::new(cc)))),
    )
}
