use autopilot::key;

pub enum KeyConvert {
    Keycode(key::Code),
    Character(key::Character),
    None,
}

/// 将字符串键码转换为 autopilot::key::KeyCode
/// 支持 device_query::Keycode 的字符串表示
pub fn pilot_key_code_from_str(key: &str) -> KeyConvert {
    // 首先尝试解析为 device_query::Keycode
    // if let Ok(keycode) = key.parse::<Keycode>() {
    //     return keycode_to_pilot_keycode(keycode);
    // }

    // 如果解析失败，尝试直接匹配字符串
    match key {
        // 数字键
        "Key0" => KeyConvert::Keycode(key::Code(key::KeyCode::Num0)),
        "Key1" => KeyConvert::Keycode(key::Code(key::KeyCode::Num1)),
        "Key2" => KeyConvert::Keycode(key::Code(key::KeyCode::Num2)),
        "Key3" => KeyConvert::Keycode(key::Code(key::KeyCode::Num3)),
        "Key4" => KeyConvert::Keycode(key::Code(key::KeyCode::Num4)),
        "Key5" => KeyConvert::Keycode(key::Code(key::KeyCode::Num5)),
        "Key6" => KeyConvert::Keycode(key::Code(key::KeyCode::Num6)),
        "Key7" => KeyConvert::Keycode(key::Code(key::KeyCode::Num7)),
        "Key8" => KeyConvert::Keycode(key::Code(key::KeyCode::Num8)),
        "Key9" => KeyConvert::Keycode(key::Code(key::KeyCode::Num9)),

        // 字母键
        "A" => KeyConvert::Character(key::Character('A')),
        "B" => KeyConvert::Character(key::Character('B')),
        "C" => KeyConvert::Character(key::Character('C')),
        "D" => KeyConvert::Character(key::Character('D')),
        "E" => KeyConvert::Character(key::Character('E')),
        "F" => KeyConvert::Character(key::Character('F')),
        "G" => KeyConvert::Character(key::Character('G')),
        "H" => KeyConvert::Character(key::Character('H')),
        "I" => KeyConvert::Character(key::Character('I')),
        "J" => KeyConvert::Character(key::Character('J')),
        "K" => KeyConvert::Character(key::Character('K')),
        "L" => KeyConvert::Character(key::Character('L')),
        "M" => KeyConvert::Character(key::Character('M')),
        "N" => KeyConvert::Character(key::Character('N')),
        "O" => KeyConvert::Character(key::Character('O')),
        "P" => KeyConvert::Character(key::Character('P')),
        "Q" => KeyConvert::Character(key::Character('Q')),
        "R" => KeyConvert::Character(key::Character('R')),
        "S" => KeyConvert::Character(key::Character('S')),
        "T" => KeyConvert::Character(key::Character('T')),
        "U" => KeyConvert::Character(key::Character('U')),
        "V" => KeyConvert::Character(key::Character('V')),
        "W" => KeyConvert::Character(key::Character('W')),
        "X" => KeyConvert::Character(key::Character('X')),
        "Y" => KeyConvert::Character(key::Character('Y')),
        "Z" => KeyConvert::Character(key::Character('Z')),

        // 功能键
        "F1" => KeyConvert::Keycode(key::Code(key::KeyCode::F1)),
        "F2" => KeyConvert::Keycode(key::Code(key::KeyCode::F2)),
        "F3" => KeyConvert::Keycode(key::Code(key::KeyCode::F3)),
        "F4" => KeyConvert::Keycode(key::Code(key::KeyCode::F4)),
        "F5" => KeyConvert::Keycode(key::Code(key::KeyCode::F5)),
        "F6" => KeyConvert::Keycode(key::Code(key::KeyCode::F6)),
        "F7" => KeyConvert::Keycode(key::Code(key::KeyCode::F7)),
        "F8" => KeyConvert::Keycode(key::Code(key::KeyCode::F8)),
        "F9" => KeyConvert::Keycode(key::Code(key::KeyCode::F9)),
        "F10" => KeyConvert::Keycode(key::Code(key::KeyCode::F10)),
        "F11" => KeyConvert::Keycode(key::Code(key::KeyCode::F11)),
        "F12" => KeyConvert::Keycode(key::Code(key::KeyCode::F12)),
        "F13" => KeyConvert::Keycode(key::Code(key::KeyCode::F13)),
        "F14" => KeyConvert::Keycode(key::Code(key::KeyCode::F14)),
        "F15" => KeyConvert::Keycode(key::Code(key::KeyCode::F15)),
        "F16" => KeyConvert::Keycode(key::Code(key::KeyCode::F16)),
        "F17" => KeyConvert::Keycode(key::Code(key::KeyCode::F17)),
        "F18" => KeyConvert::Keycode(key::Code(key::KeyCode::F18)),
        "F19" => KeyConvert::Keycode(key::Code(key::KeyCode::F19)),
        "F20" => KeyConvert::Keycode(key::Code(key::KeyCode::F20)),

        // 特殊键
        "Escape" => KeyConvert::Keycode(key::Code(key::KeyCode::Escape)),
        "Space" => KeyConvert::Keycode(key::Code(key::KeyCode::Space)),
        "Enter" => KeyConvert::Keycode(key::Code(key::KeyCode::Return)),
        "Backspace" => KeyConvert::Keycode(key::Code(key::KeyCode::Backspace)),
        "Tab" => KeyConvert::Keycode(key::Code(key::KeyCode::Tab)),
        "CapsLock" => KeyConvert::Keycode(key::Code(key::KeyCode::CapsLock)),

        // 方向键
        "Up" => KeyConvert::Keycode(key::Code(key::KeyCode::UpArrow)),
        "Down" => KeyConvert::Keycode(key::Code(key::KeyCode::DownArrow)),
        "Left" => KeyConvert::Keycode(key::Code(key::KeyCode::LeftArrow)),
        "Right" => KeyConvert::Keycode(key::Code(key::KeyCode::RightArrow)),

        // 导航键
        "Home" => KeyConvert::Keycode(key::Code(key::KeyCode::Home)),
        "End" => KeyConvert::Keycode(key::Code(key::KeyCode::End)),
        "PageUp" => KeyConvert::Keycode(key::Code(key::KeyCode::PageUp)),
        "PageDown" => KeyConvert::Keycode(key::Code(key::KeyCode::PageDown)),
        "Delete" => KeyConvert::Keycode(key::Code(key::KeyCode::Delete)),
        "Insert" => KeyConvert::None,

        // 修饰键
        "LControl" | "RControl" => KeyConvert::Keycode(key::Code(key::KeyCode::Control)),
        "LShift" | "RShift" => KeyConvert::Keycode(key::Code(key::KeyCode::Shift)),
        "LAlt" | "RAlt" => KeyConvert::Keycode(key::Code(key::KeyCode::Alt)),
        "Command" | "RCommand" | "LMeta" | "RMeta" => {
            KeyConvert::Keycode(key::Code(key::KeyCode::Meta))
        },

        // 数字键盘
        "Numpad0" => KeyConvert::Keycode(key::Code(key::KeyCode::Num0)),
        "Numpad1" => KeyConvert::Keycode(key::Code(key::KeyCode::Num1)),
        "Numpad2" => KeyConvert::Keycode(key::Code(key::KeyCode::Num2)),
        "Numpad3" => KeyConvert::Keycode(key::Code(key::KeyCode::Num3)),
        "Numpad4" => KeyConvert::Keycode(key::Code(key::KeyCode::Num4)),
        "Numpad5" => KeyConvert::Keycode(key::Code(key::KeyCode::Num5)),
        "Numpad6" => KeyConvert::Keycode(key::Code(key::KeyCode::Num6)),
        "Numpad7" => KeyConvert::Keycode(key::Code(key::KeyCode::Num7)),
        "Numpad8" => KeyConvert::Keycode(key::Code(key::KeyCode::Num8)),
        "Numpad9" => KeyConvert::Keycode(key::Code(key::KeyCode::Num9)),
        "NumpadDecimal" => KeyConvert::Keycode(key::Code(key::KeyCode::NumDecimal)),
        "NumpadEnter" => KeyConvert::Keycode(key::Code(key::KeyCode::NumEnter)),
        "NumpadAdd" => KeyConvert::Keycode(key::Code(key::KeyCode::NumAdd)),
        "NumpadSubtract" => KeyConvert::Keycode(key::Code(key::KeyCode::NumSubtract)),
        "NumpadMultiply" => KeyConvert::Keycode(key::Code(key::KeyCode::NumMultiply)),
        "NumpadDivide" => KeyConvert::Keycode(key::Code(key::KeyCode::NumDivide)),

        // 符号键
        "Grave" => KeyConvert::Character(key::Character('`')),
        "Minus" => KeyConvert::Character(key::Character('-')),
        "Equal" => KeyConvert::Character(key::Character('=')),
        "LeftBracket" => KeyConvert::Character(key::Character('[')),
        "RightBracket" => KeyConvert::Character(key::Character(']')),
        "BackSlash" => KeyConvert::Character(key::Character('\\')),
        "Semicolon" => KeyConvert::Character(key::Character(';')),
        "Apostrophe" => KeyConvert::Character(key::Character('\'')),
        "Comma" => KeyConvert::Character(key::Character(',')),
        "Dot" => KeyConvert::Character(key::Character('.')),
        "Slash" => KeyConvert::Character(key::Character('/')),

        _ => KeyConvert::None,
    }
}
