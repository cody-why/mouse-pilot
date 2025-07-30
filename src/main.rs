// hide console windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// macOS console hiding via app bundle (Info.plist with LSUIElement)

use anyhow::Result;

use eframe::egui;
use mousepilot::{font::*, ui::App};

fn main() -> Result<()> {
    mousepilot_main()
}

fn mousepilot_main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let icon = load_icon()?;

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([520.0, 500.0])
            .with_resizable(true)
            .with_position(eframe::egui::Pos2::new(100.0, 100.0))
            .with_icon(icon),
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
            if let Err(e) = config_chinese_fonts(&cc.egui_ctx) {
                autopilot::alert::alert(
                    &format!("Failed to setup Chinese fonts: {e}"),
                    Some("Alert"),
                    None,
                    None,
                );
            }

            let app = App::new(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    ) {
        autopilot::alert::alert(&format!("Failed to run app: {e}"), Some("Alert"), None, None);
    }

    Ok(())
}

pub fn load_icon() -> Result<egui::IconData> {
    use mousepilot::icon_data;
    Ok(egui::IconData {
        rgba: icon_data::get_icon_data().to_vec(),
        width: icon_data::ICON_WIDTH,
        height: icon_data::ICON_HEIGHT,
    })
}
