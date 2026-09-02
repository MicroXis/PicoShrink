mod app;
mod compression;
mod ghostscript;
mod models;
mod theme;

use app::PdfCompressorApp;
use eframe::egui;

fn load_icon() -> egui::IconData {
    let icon_bytes = include_bytes!("../assets/icon-256.png");

    let image = image::load_from_memory(icon_bytes)
        .expect("Impossible de charger l'icône")
        .into_rgba8();

    let (width, height) = image.dimensions();

    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "PDF Compressor",
        options,
        Box::new(|cc| {
            theme::configure(&cc.egui_ctx);
            Ok(Box::new(PdfCompressorApp::default()))
        }),
    )
}
