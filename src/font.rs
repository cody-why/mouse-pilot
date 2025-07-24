use eframe::egui;
use std::{env, fs};

pub fn load_default_font(ctx: &egui::Context) {
    #[cfg(target_os = "windows")]
    load_windows_font(ctx);

    #[cfg(target_os = "macos")]
    load_macos_font(ctx);
}

#[allow(unused)]
fn load_macos_font(ctx: &egui::Context) {
    // let font_path = "/System/Library/Fonts/STHeiti Light.ttc";
    let font_path = "/System/Library/Fonts/Supplemental/Arial Unicode.ttf";
    load_font(ctx, font_path);
}

#[allow(unused)]
fn load_windows_font(ctx: &egui::Context) {
    let system_root = env::var("SystemRoot").expect("无法获取系统根目录");
    let font_path = format!("{system_root}\\Fonts\\msyh.ttc"); // 微软雅黑

    load_font(ctx, &font_path);
}

pub fn load_font(ctx: &egui::Context, font_path: &str) {
    let mut fonts = egui::FontDefinitions::default();

    let font_data = fs::read(font_path).expect("无法加载字体文件");
    // 加载中文字体
    fonts
        .font_data
        .insert("my_font".to_owned(), egui::FontData::from_owned(font_data).into());

    // 设置为比例字体和等宽字体的优先选项
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "my_font".to_owned());
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("my_font".to_owned());

    ctx.set_fonts(fonts);
}
