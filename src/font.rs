use eframe::egui;

#[derive(Debug)]
pub enum FontError {
    NotFound(String),
    ReadError(std::io::Error),
    UnsupportedPlatform,
}

impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontError::NotFound(path) => write!(f, "Font file not found: {path}"),
            FontError::ReadError(err) => write!(f, "Failed to read font file: {err}"),
            FontError::UnsupportedPlatform => write!(f, "Platform not supported"),
        }
    }
}

impl std::error::Error for FontError {}

pub fn config_chinese_fonts(ctx: &egui::Context) -> Result<(), FontError> {
    let font_data = load_chinese_font()?;
    let mut fonts = egui::FontDefinitions::default();

    // Insert the Chinese font
    fonts.font_data.insert("chinese".to_owned(), font_data.into());

    // Configure font families
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "chinese".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "chinese".to_owned());

    // Apply the font configuration
    ctx.set_fonts(fonts);

    config_style(ctx);

    Ok(())
}

pub fn config_style(ctx: &egui::Context) {
    ctx.style_mut_of(egui::Theme::Light, |style| {
        let color = egui::Color32::from_rgb(0, 0, 0);
        // 设置文本颜色
        style.visuals.override_text_color = Some(color);
        // 设置按钮颜色
        style.visuals.widgets.inactive.fg_stroke.color = color;
        // #1e88dd
        style.visuals.widgets.hovered.bg_stroke.color = egui::Color32::from_rgb(30, 136, 221);
        style.visuals.widgets.inactive.bg_stroke.width = 1.0;
        // #b3b3b3
        // #e6e6e6
        style.visuals.widgets.inactive.bg_stroke.color = egui::Color32::from_rgb(230, 230, 230);
    });

    ctx.style_mut_of(egui::Theme::Dark, |style| {
        let color = egui::Color32::from_rgb(183, 183, 183);
        style.visuals.override_text_color = Some(color);
        // 设置按钮颜色
        style.visuals.widgets.inactive.fg_stroke.color = color;
        // #1e88dd
        style.visuals.widgets.hovered.bg_stroke.color = egui::Color32::from_rgb(30, 136, 221);
    });
}

#[cfg(target_os = "windows")]
fn load_windows_chinese_font() -> Result<egui::FontData, FontError> {
    let system_path = std::env::var("SYSTEMROOT").unwrap_or("C:\\Windows".to_string());
    let font_paths = [
        format!("{system_path}\\Fonts\\msyh.ttc"), // Microsoft YaHei
        format!("{system_path}\\Fonts\\simhei.ttf"), // SimHei
        format!("{system_path}\\Fonts\\simsun.ttc"), // SimSun
        format!("{system_path}\\Fonts\\simkai.ttf"), // KaiTi
        format!("{system_path}\\Fonts\\simfang.ttf"), // FangSong
        format!("{system_path}\\Fonts\\msjh.ttc"), // Microsoft JhengHei (Traditional Chinese)
        format!("{system_path}\\Fonts\\kaiu.ttf"), // DFKai-SB (Traditional Chinese)
        format!("{system_path}\\Fonts\\mingliu.ttc"), // MingLiU (Traditional Chinese)
    ];

    for font_path in &font_paths {
        if let Ok(font_data) = load_font(font_path) {
            return Ok(font_data);
        }
    }

    Err(FontError::NotFound("No Chinese font found".to_string()))
}

#[cfg(target_os = "macos")]
fn load_macos_chinese_font() -> Result<egui::FontData, FontError> {
    let font_paths = [
        "/System/Library/Fonts/PingFang.ttc",      // PingFang SC
        "/System/Library/Fonts/STHeiti Light.ttc", // STHeiti
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc", // Hiragino Sans GB
        "/Library/Fonts/Arial Unicode.ttf",           // Arial Unicode MS
        "/System/Library/Fonts/Apple LiGothic Medium.ttf", // Apple LiGothic (Traditional)
    ];

    for font_path in font_paths {
        if let Ok(font_data) = load_font(font_path) {
            return Ok(font_data);
        }
    }

    Err(FontError::NotFound("No Chinese font found".to_string()))
}

#[inline]
fn load_font(font_path: &str) -> Result<egui::FontData, FontError> {
    use std::io::Read;

    let file = std::fs::File::open(font_path).map_err(FontError::ReadError)?;
    let mut reader = std::io::BufReader::new(file);
    let mut font_data = Vec::new();
    reader.read_to_end(&mut font_data).map_err(FontError::ReadError)?;
    Ok(egui::FontData::from_owned(font_data))
}

#[cfg(target_os = "linux")]
fn load_linux_chinese_font() -> Result<egui::FontData, FontError> {
    // Common Chinese font paths on Linux distributions
    let font_paths = [
        "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        "/usr/share/fonts/truetype/arphic/uming.ttc",
        "/usr/share/fonts/truetype/arphic/ukai.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        // Ubuntu/Debian paths
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        // CentOS/RHEL paths
        "/usr/share/fonts/google-droid/DroidSansFallbackFull.ttf",
        // Arch Linux paths
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    ];

    for font_path in font_paths {
        if let Ok(font_data) = load_font(font_path) {
            return Ok(font_data);
        }
    }

    Err(FontError::NotFound("Chinese font not found".to_string()))
}

fn load_chinese_font() -> Result<egui::FontData, FontError> {
    #[cfg(target_os = "windows")]
    {
        load_windows_chinese_font()
    }

    #[cfg(target_os = "macos")]
    {
        load_macos_chinese_font()
    }

    #[cfg(target_os = "linux")]
    {
        load_linux_chinese_font()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(FontError::UnsupportedPlatform)
    }
}
