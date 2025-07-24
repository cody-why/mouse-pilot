#![allow(clippy::new_without_default)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use anyhow::Result;

use mousepilot::{font::load_default_font, ui::App};

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([500.0, 500.0])
            .with_resizable(true),
        // .with_transparent(true),
        // renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "录制器",
        native_options,
        Box::new(|_cc| {
            // 配置字体以支持中文显示
            load_default_font(&_cc.egui_ctx);
            let app = App::new();
            Ok(Box::new(app))
        }),
    ) {
        autopilot::alert::alert(&format!("Failed to run app: {e}"), None, None, None);
    }

    Ok(())
}
