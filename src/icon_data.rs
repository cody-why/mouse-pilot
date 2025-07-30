pub const ICON_WIDTH: u32 = 256;
pub const ICON_HEIGHT: u32 = 256;

pub fn get_icon_data() -> &'static [u8] {
    include_bytes!("../assets/icon_data.bin")
}
