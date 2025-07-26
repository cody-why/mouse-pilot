#![allow(clippy::new_without_default)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use anyhow::Result;

use mousepilot::{font::setup_chinese_fonts, ui::App};

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([520.0, 500.0])
            .with_resizable(true)
            .with_position(eframe::egui::Pos2::new(100.0, 100.0)), // 窗口位置

        renderer: eframe::Renderer::Glow,

        // 硬件加速设置
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,

        vsync: true,

        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "鼠标录制器",
        native_options,
        Box::new(|cc| {
            // 配置字体以支持中文显示
            if let Err(e) = setup_chinese_fonts(&cc.egui_ctx) {
                autopilot::alert::alert(
                    &format!("Failed to setup Chinese fonts: {e}"),
                    Some("Alert"),
                    None,
                    None,
                );
            }

            let app = App::new();
            Ok(Box::new(app))
        }),
    ) {
        autopilot::alert::alert(&format!("Failed to run app: {e}"), Some("Alert"), None, None);
    }

    Ok(())
}
