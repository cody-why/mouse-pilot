use anyhow::Result;
use device_query::{DeviceQuery, DeviceState, Keycode, MouseState};
use eframe::egui;
use parking_lot::Mutex;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use crate::{event::*, hotkey::Shortcut};

#[derive(Debug, Clone)]
pub struct MacroRecorder {
    events: Arc<Mutex<Vec<MacroEvent>>>,
    is_recording: Arc<AtomicBool>,
    start_time: Arc<Mutex<Option<Instant>>>,
    last_mouse_pos: Arc<Mutex<(i32, i32)>>,
    // 添加设备状态监听
    device_state: Arc<Mutex<DeviceState>>,
    shortcuts: Arc<Vec<Shortcut>>,
    click_time: Arc<Mutex<Option<Instant>>>,
}

impl MacroRecorder {
    pub fn new(shortcuts: Arc<Vec<Shortcut>>) -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            is_recording: Arc::new(AtomicBool::new(false)),
            start_time: Arc::new(Mutex::new(None)),
            last_mouse_pos: Arc::new(Mutex::new((0, 0))),
            device_state: Arc::new(Mutex::new(DeviceState::new())),
            shortcuts,
            click_time: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_recording(&self) -> Result<()> {
        if self.is_recording.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.is_recording.store(true, Ordering::SeqCst);
        *self.start_time.lock() = Some(Instant::now());
        self.events.lock().clear();

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

                const MIN_DIST: i32 = 16;
                let lastpos = *recorder.last_mouse_pos.lock();
                // 监听鼠标事件
                let mouse_state = device_state.lock().get_mouse();
                // if mouse_state.coords != last_mouse_state.coords {
                //     recorder.add_mouse_move(mouse_state.coords.0, mouse_state.coords.1);
                // }

                // 监听鼠标点击
                if mouse_state.button_pressed != last_mouse_state.button_pressed {
                    if mouse_state.coords != lastpos {
                        recorder.add_mouse_move(mouse_state.coords.0, mouse_state.coords.1);
                    }
                    for (i, pressed) in mouse_state.button_pressed.iter().enumerate() {
                        if *pressed {
                            recorder.add_mouse_click(
                                Button::from(i),
                                true,
                                // mouse_state.coords.0,
                                // mouse_state.coords.1,
                            );
                        } else if *last_mouse_state.button_pressed.get(i).unwrap_or(&false) {
                            recorder.add_mouse_click(
                                Button::from(i),
                                false,
                                // mouse_state.coords.0,
                                // mouse_state.coords.1,
                            );
                        }
                    }
                // }
                } else {
                    let (cur_x, cur_y) = mouse_state.coords;
                    if (cur_x - lastpos.0).abs() >= MIN_DIST
                        || (cur_y - lastpos.1).abs() >= MIN_DIST
                    {
                        recorder.add_mouse_move(cur_x, cur_y);
                    }
                }

                // 监听键盘事件
                let keys = device_state.lock().get_keys();
                // 排除快捷键
                if !recorder.is_hotkey(&keys) {
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
                }

                last_mouse_state = mouse_state;
                last_keys = keys;

                std::thread::sleep(Duration::from_millis(10));
            }
        });

        Ok(())
    }

    pub fn stop_recording(&self) {
        if !self.is_recording.load(Ordering::SeqCst) {
            return;
        }

        self.is_recording.store(false, Ordering::SeqCst);
        *self.start_time.lock() = None;
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    pub fn get_time_elapsed(&self) -> u64 {
        (*self.start_time.lock())
            .map(|time| time.elapsed().as_millis() as u64)
            .unwrap_or(0)
    }

    pub fn get_click_time_elapsed(&self) -> u64 {
        (*self.click_time.lock())
            .map(|time| time.elapsed().as_millis() as u64)
            .unwrap_or(0)
    }

    pub fn get_events(&self) -> Vec<MacroEvent> {
        self.events.lock().clone()
    }

    pub fn get_event_count(&self) -> usize {
        self.events.lock().len()
    }

    // 手动添加事件的方法
    pub fn add_mouse_move(&self, x: i32, y: i32) {
        if !self.is_recording.load(Ordering::SeqCst) {
            return;
        }

        let timestamp = match *self.start_time.lock() {
            Some(time) => time.elapsed().as_millis(),
            None => return,
        };

        // 更新最后鼠标位置
        *self.last_mouse_pos.lock() = (x, y);

        let macro_event = MacroEvent {
            event_type: MacroEventType::MouseMove { x, y },
            timestamp,
        };

        self.events.lock().push(macro_event);
        // self.start_time.lock().replace(Instant::now());
    }

    pub fn add_mouse_click(&self, button: Button, pressed: bool) {
        if !self.is_recording.load(Ordering::SeqCst) {
            return;
        }

        let timestamp = match *self.start_time.lock() {
            Some(time) => time.elapsed().as_millis(),
            None => return,
        };

        let macro_event = MacroEvent {
            event_type: MacroEventType::MouseClick {
                button,
                pressed,
                // x,
                // y,
            },
            timestamp,
        };

        self.events.lock().push(macro_event);
        // self.start_time.lock().replace(Instant::now());
        self.click_time.lock().replace(Instant::now());
    }

    fn is_hotkey(&self, keys: &[Keycode]) -> bool {
        for key in keys {
            if let Some(key) = egui::Key::from_name(&key.to_string()) {
                for shortcut in self.shortcuts.iter() {
                    if shortcut.key == key {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn add_key_event(&self, key: &str, pressed: bool) {
        if !self.is_recording.load(Ordering::SeqCst) {
            return;
        }

        let timestamp = match *self.start_time.lock() {
            Some(time) => time.elapsed().as_millis(),
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
            timestamp,
        };

        self.events.lock().push(macro_event);
        // self.start_time.lock().replace(Instant::now());
    }

    // 新增延时事件方法
    pub fn add_delay(&self, duration_ms: u64) {
        if !self.is_recording.load(Ordering::SeqCst) {
            return;
        }

        let timestamp = match *self.start_time.lock() {
            Some(time) => time.elapsed().as_millis(),
            None => return,
        };

        let macro_event = MacroEvent {
            event_type: MacroEventType::Delay { duration_ms },
            timestamp,
        };
        self.events.lock().push(macro_event);
    }

    pub fn clear_events(&self) {
        self.events.lock().clear();
    }
}
