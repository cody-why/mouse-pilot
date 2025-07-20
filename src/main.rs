#![allow(clippy::new_without_default)]
use anyhow::Result;

use mousepilot::ui::App;

fn main() -> Result<()> {
    env_logger::init();

    let app = App::new();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "鼠标键盘宏录制器",
        native_options,
        Box::new(|cc| -> Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>> {
            // 配置字体以支持中文显示
            App::configure_fonts(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run app: {}", e))?;

    Ok(())
}
