#[cfg(test)]
mod tests {
    use image::ImageReader;
    use std::fs;
    // 转换 icon 图片为 Rust 代码, 避免引入 image 依赖
    #[test]
    fn convert_icon() {
        let input_path = "assets/icon.png";
        let output_path = "src/icon_data.rs";
        // 读取并转换图片
        let img = ImageReader::open(input_path).unwrap().decode().unwrap().into_rgba8();

        let (width, height) = img.dimensions();
        let rgba_data = img.into_raw();
        fs::write("assets/icon_data.bin", rgba_data).unwrap();

        // 生成 Rust 代码
        let mut code = String::new();
        code.push_str("pub const ICON_WIDTH: u32 = ");
        code.push_str(&width.to_string());
        code.push_str(";\npub const ICON_HEIGHT: u32 = ");
        code.push_str(&height.to_string());
        code.push_str(";\n\n");
        code.push_str("pub fn get_icon_data() -> &'static [u8] {\n");
        code.push_str("    include_bytes!(\"../assets/icon_data.bin\")");
        code.push_str("}\n");

        // 写入文件
        fs::write(output_path, code).unwrap();
    }
}
