use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use device_query::{DeviceQuery, DeviceState, Keycode};
use eframe::egui;
use log::debug;
use parking_lot::Mutex;

use crate::state::AppState;

// 快捷键结构体
#[derive(Debug, Clone)]
pub struct Shortcut {
    pub name: String,
    pub key: egui::Key,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub description: String,
    pub is_ui: bool,
}

impl Shortcut {
    pub fn new(
        name: &str, key: egui::Key, ctrl: bool, shift: bool, alt: bool, description: &str,
        is_ui: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            key,
            ctrl,
            shift,
            alt,
            description: description.to_string(),
            is_ui,
        }
    }

    /// 检查快捷键是否匹配UI快捷键
    pub fn matches(&self, key: egui::Key, modifiers: &egui::Modifiers) -> bool {
        if !self.is_ui {
            return false;
        }
        if key != self.key {
            return false;
        }

        if self.ctrl && !modifiers.ctrl {
            return false;
        }
        if self.shift && !modifiers.shift {
            return false;
        }
        if self.alt && !modifiers.alt {
            return false;
        }

        true
    }

    /// 检查快捷键是否匹配全局快捷键
    pub fn matches_keycode(&self, key: &egui::Key, keys: &[Keycode]) -> bool {
        if self.is_ui {
            return false;
        }
        if *key != self.key {
            return false;
        }
        // 检查修饰键
        if self.ctrl && !(keys.contains(&Keycode::LControl) || keys.contains(&Keycode::RControl)) {
            return false;
        }
        if self.shift && !(keys.contains(&Keycode::LShift) || keys.contains(&Keycode::RShift)) {
            return false;
        }
        if self.alt && !(keys.contains(&Keycode::LAlt) || keys.contains(&Keycode::RAlt)) {
            return false;
        }

        true
    }

    /// 将device_query::Keycode转换为egui::Key
    pub fn to_key(keycode: &Keycode) -> Option<egui::Key> {
        egui::Key::from_name(&keycode.to_string())
    }

    /// 将egui::Key转换为device_query::Keycode
    pub fn to_keycode(&self) -> Option<Keycode> {
        use std::str::FromStr;
        Keycode::from_str(self.key.name()).ok()
    }

    pub fn display_text(&self) -> String {
        let mut parts = Vec::new();

        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }

        parts.push(self.key.name().to_string());

        parts.join(" + ")
    }
}

pub struct GlobalHotkeyListener {
    running: Arc<AtomicBool>,
    listener_task: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl GlobalHotkeyListener {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            listener_task: Arc::new(parking_lot::Mutex::new(None)),
        }
    }

    pub fn start(&self, state: Arc<AppState>) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let listener_task = self.listener_task.clone();

        let handle = thread::spawn(move || {
            Self::run_listener_loop(running, state);
        });

        *listener_task.lock() = Some(handle);
    }

    fn run_listener_loop(running: Arc<AtomicBool>, state: Arc<AppState>) {
        let device_state = DeviceState::new();
        let mut last_keys = Vec::new();

        while running.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_millis(10));

            let keys = device_state.get_keys();
            if keys != last_keys {
                for key in keys.iter() {
                    if let Some(key) = Shortcut::to_key(key) {
                        for shortcut in state.shortcuts.iter() {
                            if shortcut.matches_keycode(&key, &keys) {
                                debug!("检测到全局快捷键: {}", shortcut.name);
                                ShortcutProcessor::execute_shortcut(&shortcut.name, &state);
                                break;
                            }
                        }
                    }
                }
                last_keys = keys;
            }
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);

        if let Some(_handle) = self.listener_task.lock().take() {
            // handle.abort();
        }
    }
}

pub struct ShortcutProcessor;

impl ShortcutProcessor {
    pub fn execute_shortcut(shortcut_name: &str, state: &AppState) {
        debug!("执行全局快捷键: {shortcut_name}");
        match shortcut_name {
            "start_recording" => {
                state.stop_player();
                if let Err(e) = state.recorder.start_recording() {
                    debug!("Failed to start recording: {e}");
                }
                state.ui_repaint_after_secs(0.2);
            },
            "stop" => {
                if state.recorder.is_recording() {
                    state.recorder.stop_recording();
                }
                if state.is_playing() {
                    state.stop_player();
                }
            },
            "play_once" => {
                if !state.recorder.is_recording() {
                    state.play_selected_macros(1);
                }
            },
            "play_multiple" => {
                if !state.recorder.is_recording() {
                    state.play_selected_macros(state.get_repeat_count());
                }
            },
            "clear_recording" => {
                state.recorder.clear_events();
            },
            _ => {
                debug!("未知的快捷键: {shortcut_name}");
            },
        }
    }
}
