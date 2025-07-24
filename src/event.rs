use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroEvent {
    pub event_type: MacroEventType,
    pub timestamp: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroEventType {
    MouseMove {
        x: i32,
        y: i32,
    },
    MouseClick {
        button: Button,
        pressed: bool,
        // x: i32,
        // y: i32,
    },
    KeyPress {
        key: String,
    },
    KeyRelease {
        key: String,
    },
    // 新增图像识别事件
    ImageFind {
        image_path: String,
        confidence: f64,
        timeout: u64,
    },

    // 新增延时事件
    Delay {
        duration_ms: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Button {
    Left,
    Middle,
    Right,
}

impl From<usize> for Button {
    fn from(button: usize) -> Self {
        match button {
            1 => Button::Left,
            2 => Button::Right,
            3 => Button::Middle,
            _ => Button::Left,
        }
    }
}
