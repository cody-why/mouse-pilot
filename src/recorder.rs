use anyhow::Result;
use device_query::{DeviceQuery, DeviceState, MouseState};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use crate::event::*;

#[derive(Debug, Clone)]
pub struct MacroRecorder {
    events: Arc<Mutex<Vec<MacroEvent>>>,
    is_recording: Arc<AtomicBool>,
    start_time: Arc<Mutex<Option<Instant>>>,
    last_mouse_pos: Arc<Mutex<(i32, i32)>>,
    // 添加设备状态监听
    device_state: Arc<Mutex<DeviceState>>,
}

impl MacroRecorder {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            is_recording: Arc::new(AtomicBool::new(false)),
            start_time: Arc::new(Mutex::new(None)),
            last_mouse_pos: Arc::new(Mutex::new((0, 0))),
            device_state: Arc::new(Mutex::new(DeviceState::new())),
        }
    }

    pub fn start_recording(&self) -> Result<()> {
        if self.is_recording.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.is_recording.store(true, Ordering::SeqCst);
        *self.start_time.lock().unwrap() = Some(Instant::now());
        self.events.lock().unwrap().clear();

        // 启动设备监听线程
        let recorder = self.clone();
        let is_recording = self.is_recording.clone();
        let device_state = self.device_state.clone();

        std::thread::spawn(move || {
            let mut last_mouse_state = MouseState::default();
            let mut last_keys = Vec::new();

            loop {
                if !is_recording.load(Ordering::SeqCst) {
                    break;
                }

                // 监听鼠标事件
                let mouse_state = device_state.lock().unwrap().get_mouse();
                if mouse_state.coords != last_mouse_state.coords {
                    recorder.add_mouse_move(mouse_state.coords.0, mouse_state.coords.1);
                }

                // 监听鼠标点击 - 使用更安全的方法
                let current_left_pressed = mouse_state.button_pressed.first().unwrap_or(&false);
                let last_left_pressed = last_mouse_state.button_pressed.first().unwrap_or(&false);

                if *current_left_pressed && !*last_left_pressed {
                    recorder.add_mouse_click(
                        "Left",
                        true,
                        mouse_state.coords.0,
                        mouse_state.coords.1,
                    );
                }
                if !*current_left_pressed && *last_left_pressed {
                    recorder.add_mouse_click(
                        "Left",
                        false,
                        mouse_state.coords.0,
                        mouse_state.coords.1,
                    );
                }

                // 监听键盘事件
                let keys = device_state.lock().unwrap().get_keys();
                for key in &keys {
                    if !last_keys.contains(key) {
                        recorder.add_key_event(&format!("{key:?}"), true);
                    }
                }
                for key in &last_keys {
                    if !keys.contains(key) {
                        recorder.add_key_event(&format!("{key:?}"), false);
                    }
                }

                last_mouse_state = mouse_state;
                last_keys = keys;

                std::thread::sleep(Duration::from_millis(1));
            }
        });

        Ok(())
    }

    pub fn stop_recording(&self) {
        self.is_recording.store(false, Ordering::SeqCst);
        *self.start_time.lock().unwrap() = None;
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    pub fn get_events(&self) -> Vec<MacroEvent> {
        self.events.lock().unwrap().clone()
    }

    pub fn get_events_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    // 手动添加事件的方法
    pub fn add_mouse_move(&self, x: i32, y: i32) {
        if !self.is_recording.load(Ordering::SeqCst) {
            return;
        }

        let start_time = match *self.start_time.lock().unwrap() {
            Some(time) => time,
            None => return,
        };

        // 更新最后鼠标位置
        if let Ok(mut pos) = self.last_mouse_pos.lock() {
            *pos = (x, y);
        }

        let macro_event = MacroEvent {
            event_type: MacroEventType::MouseMove { x, y },
            timestamp: start_time.elapsed().as_millis(),
        };

        if let Ok(mut events) = self.events.lock() {
            events.push(macro_event);
        }
    }

    pub fn add_mouse_click(&self, button: &str, pressed: bool, x: i32, y: i32) {
        if !self.is_recording.load(Ordering::SeqCst) {
            return;
        }

        let start_time = match *self.start_time.lock().unwrap() {
            Some(time) => time,
            None => return,
        };

        let macro_event = MacroEvent {
            event_type: MacroEventType::MouseClick {
                button: button.to_string(),
                pressed,
                x,
                y,
            },
            timestamp: start_time.elapsed().as_millis(),
        };

        if let Ok(mut events) = self.events.lock() {
            events.push(macro_event);
        }
    }

    pub fn add_key_event(&self, key: &str, pressed: bool) {
        if !self.is_recording.load(Ordering::SeqCst) {
            return;
        }

        let start_time = match *self.start_time.lock().unwrap() {
            Some(time) => time,
            None => return,
        };

        let macro_event = MacroEvent {
            event_type: if pressed {
                MacroEventType::KeyPress {
                    key: key.to_string(),
                }
            } else {
                MacroEventType::KeyRelease {
                    key: key.to_string(),
                }
            },
            timestamp: start_time.elapsed().as_millis(),
        };

        if let Ok(mut events) = self.events.lock() {
            events.push(macro_event);
        }
    }

    // 新增图像识别相关方法
    pub fn add_image_find(&self, image_path: &str, confidence: f64, timeout: u64) {
        if !self.is_recording.load(Ordering::SeqCst) {
            return;
        }

        let start_time = match *self.start_time.lock().unwrap() {
            Some(time) => time,
            None => return,
        };

        let macro_event = MacroEvent {
            event_type: MacroEventType::ImageFind {
                image_path: image_path.to_string(),
                confidence,
                timeout,
            },
            timestamp: start_time.elapsed().as_millis(),
        };

        if let Ok(mut events) = self.events.lock() {
            events.push(macro_event);
        }
    }

    pub fn add_wait_for_image(&self, image_path: &str, confidence: f64, timeout: u64) {
        if !self.is_recording.load(Ordering::SeqCst) {
            return;
        }

        let start_time = match *self.start_time.lock().unwrap() {
            Some(time) => time,
            None => return,
        };

        let macro_event = MacroEvent {
            event_type: MacroEventType::WaitForImage {
                image_path: image_path.to_string(),
                confidence,
                timeout,
            },
            timestamp: start_time.elapsed().as_millis(),
        };

        if let Ok(mut events) = self.events.lock() {
            events.push(macro_event);
        }
    }

    pub fn add_screenshot(&self, path: &str) {
        if !self.is_recording.load(Ordering::SeqCst) {
            return;
        }

        let start_time = match *self.start_time.lock().unwrap() {
            Some(time) => time,
            None => return,
        };

        let macro_event = MacroEvent {
            event_type: MacroEventType::Screenshot {
                path: path.to_string(),
            },
            timestamp: start_time.elapsed().as_millis(),
        };

        if let Ok(mut events) = self.events.lock() {
            events.push(macro_event);
        }
    }
}
