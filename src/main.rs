#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

mod appearance;
mod catalog;
mod config;
mod launcher;

fn main() -> eframe::Result {
    let logo = include_bytes!("../assets/sunrise-logo.png");
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 850.0])
        .with_min_inner_size([650.0, 475.0]);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(logo) {
        viewport = viewport.with_icon(icon);
    }
    eframe::run_native(
        "Sunrise Launcher",
        eframe::NativeOptions {
            viewport,
            renderer: eframe::Renderer::Wgpu,
            ..Default::default()
        },
        Box::new(|cc| {
            let path = dirs::config_dir()
                .map(|dir| dir.join("Sunrise Launcher Local").join("settings.json"));
            Ok(Box::new(launcher::Launcher::new(&cc.egui_ctx, path)))
        }),
    )
}
