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
        button: String,
        pressed: bool,
        x: i32,
        y: i32,
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
    WaitForImage {
        image_path: String,
        confidence: f64,
        timeout: u64,
    },
    Screenshot {
        path: String,
    },
}
