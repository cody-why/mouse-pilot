use anyhow::Result;
use device_query::{DeviceQuery, DeviceState, Keycode, MouseState};

use parking_lot::Mutex;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use crate::{event::*, hotkey::Shortcut};

#[derive(Debug, Clone)]
pub struct MacroRecorder {
    events: Arc<Mutex<Vec<MacroEvent>>>,
    is_recording: Arc<AtomicBool>,
    start_time: Arc<Mutex<Option<Instant>>>,
    // last_mouse_pos: Arc<Mutex<(i32, i32)>>,
    recording_task: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    shortcuts: Arc<Vec<Shortcut>>,
    click_time: Arc<Mutex<Option<Instant>>>,
}

impl MacroRecorder {
    pub fn new(shortcuts: Arc<Vec<Shortcut>>) -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            is_recording: Arc::new(AtomicBool::new(false)),
            start_time: Arc::new(Mutex::new(None)),
            // last_mouse_pos: Arc::new(Mutex::new((0, 0))),
            recording_task: Arc::new(Mutex::new(None)),
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

        // 启动异步录制任务
        let recorder = self.clone();
        let is_recording = self.is_recording.clone();

        let handle = thread::spawn(move || {
            recorder.run_recording_loop(is_recording);
        });

        *self.recording_task.lock() = Some(handle);

        Ok(())
    }

    fn run_recording_loop(&self, is_recording: Arc<AtomicBool>) {
        let device_state = DeviceState::new();
        let mut last_mouse_state = MouseState::default();
        let mut last_keys = Vec::new();

        while is_recording.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(10));

            // const MIN_DIST: i32 = 8;
            // let lastpos = *self.last_mouse_pos.lock();

            // 监听鼠标事件
            let mouse_state = device_state.get_mouse();
            if mouse_state.coords != last_mouse_state.coords {
                self.add_mouse_move(mouse_state.coords.0, mouse_state.coords.1);
            }

            // 监听鼠标点击
            if mouse_state.button_pressed != last_mouse_state.button_pressed {
                // if mouse_state.coords != last_mouse_state.coords {
                //     self.add_mouse_move(mouse_state.coords.0, mouse_state.coords.1);
                // }
                for (i, pressed) in mouse_state.button_pressed.iter().enumerate() {
                    if *pressed {
                        self.add_mouse_click(Button::from(i), true);
                    } else if *last_mouse_state.button_pressed.get(i).unwrap_or(&false) {
                        self.add_mouse_click(Button::from(i), false);
                    }
                }
            }
            // } else {
            //     let (cur_x, cur_y) = mouse_state.coords;
            //     if (cur_x - lastpos.0).abs() >= MIN_DIST || (cur_y - lastpos.1).abs() >= MIN_DIST {
            //         self.add_mouse_move(cur_x, cur_y);
            //     }
            // }

            // 监听键盘事件
            let keys = device_state.get_keys();
            if keys != last_keys {
                for key in &keys {
                    if !last_keys.contains(key) {
                        self.add_key_event(&key.to_string(), true);
                    }
                }
                for key in &last_keys {
                    if !keys.contains(key) {
                        self.add_key_event(&key.to_string(), false);
                    }
                }
                last_keys = keys;
            }

            last_mouse_state = mouse_state;
        }
    }

    pub fn stop_recording(&self) {
        self.is_recording.store(false, Ordering::SeqCst);

        if let Some(_handle) = self.recording_task.lock().take() {
            // handle.abort();
        }
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

    pub fn add_mouse_move(&self, x: i32, y: i32) {
        let elapsed = self.get_time_elapsed();
        let event = MacroEvent {
            event_type: MacroEventType::MouseMove { x, y },
            timestamp: elapsed as u128,
        };
        self.events.lock().push(event);
        // *self.last_mouse_pos.lock() = (x, y);
    }

    pub fn add_mouse_click(&self, button: Button, pressed: bool) {
        let elapsed = self.get_time_elapsed();
        let event = MacroEvent {
            event_type: MacroEventType::MouseClick { button, pressed },
            timestamp: elapsed as u128,
        };
        self.events.lock().push(event);

        if pressed {
            *self.click_time.lock() = Some(Instant::now());
        }
    }

    fn is_hotkey(&self, keys: &[Keycode]) -> bool {
        for shortcut in self.shortcuts.iter() {
            if shortcut.matches_keycode(&shortcut.key, keys) {
                return true;
            }
        }
        false
    }

    pub fn add_key_event(&self, key: &str, pressed: bool) {
        // 检查是否为快捷键
        if let Ok(keycode) = key.parse::<Keycode>() {
            let keys = vec![keycode];
            if self.is_hotkey(&keys) {
                return; // 跳过快捷键事件
            }
        }

        let elapsed = self.get_time_elapsed();
        let event = MacroEvent {
            event_type: if pressed {
                MacroEventType::KeyPress {
                    key: key.to_string(),
                }
            } else {
                MacroEventType::KeyRelease {
                    key: key.to_string(),
                }
            },
            timestamp: elapsed as u128,
        };
        self.events.lock().push(event);
    }

    pub fn add_delay(&self, duration_ms: u64) {
        let elapsed = self.get_time_elapsed();
        let event = MacroEvent {
            event_type: MacroEventType::Delay { duration_ms },
            timestamp: elapsed as u128,
        };
        self.events.lock().push(event);
    }

    pub fn clear_events(&self) {
        self.events.lock().clear();
        *self.start_time.lock() = None;
        *self.click_time.lock() = None;
    }
}
