mod app;
pub mod hash;
mod theme;

/// Launch the native AdlerIt desktop window.
pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("AdlerIt")
            .with_inner_size([520.0, 360.0])
            .with_min_inner_size([420.0, 320.0]),
        ..Default::default()
    };

    eframe::run_native(
        "AdlerIt",
        options,
        Box::new(|cc| Ok(Box::new(app::AdlerApp::new(cc)))),
    )
}
