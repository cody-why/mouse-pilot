use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use device_query::{DeviceQuery, DeviceState, Keycode};
use eframe::egui;
use log::debug;

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
    pub fn to_key(keycode: Keycode) -> Option<egui::Key> {
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

        parts.push(format!("{:?}", self.key));
        parts.join("+")
    }
}

// 全局快捷键监听器
pub struct GlobalHotkeyListener {
    device_state: DeviceState,
    running: Arc<AtomicBool>,
    shortcuts: Vec<Shortcut>,
}

impl GlobalHotkeyListener {
    pub fn new(shortcuts: Vec<Shortcut>) -> Self {
        Self {
            device_state: DeviceState::new(),
            running: Arc::new(AtomicBool::new(true)),
            shortcuts,
        }
    }

    pub fn start(&self, state: Arc<AppState>) {
        let running = self.running.clone();
        let shortcuts = self.shortcuts.clone();
        let device_state = self.device_state.clone();
        std::thread::spawn(move || {
            let mut last_keys = Vec::new();
            while running.load(Ordering::SeqCst) {
                let keys = device_state.get_keys();
                for key in &keys {
                    if !last_keys.contains(key) {
                        if let Some(key) = &Shortcut::to_key(*key) {
                            for shortcut in &shortcuts {
                                if shortcut.matches_keycode(key, &keys) {
                                    // 直接处理
                                    ShortcutProcessor::execute_shortcut(&shortcut.name, &state);
                                    std::thread::sleep(std::time::Duration::from_millis(1));
                                }
                            }
                        }
                    }
                }
                last_keys = keys;
                std::thread::sleep(std::time::Duration::from_millis(16)); // 60Hz
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

// 只保留静态方法
pub struct ShortcutProcessor;

impl ShortcutProcessor {
    pub fn execute_shortcut(shortcut_name: &str, state: &AppState) {
        debug!("独立线程执行快捷键: {shortcut_name}");
        match shortcut_name {
            "start_recording" => {
                state.stop_player();
                if let Err(e) = state.recorder.start_recording() {
                    debug!("Failed to start recording: {e}");
                }
            },
            "stop_recording" => {
                state.recorder.stop_recording();
            },
            "clear_recording" => {
                state.recorder.clear_events();
            },
            "stop_playback" => {
                state.stop_player();
            },
            "play_once" => {
                let selected = state.get_selected_macros();
                if !selected.is_empty() && !state.recorder.is_recording() {
                    let interval = state.get_macro_interval_ms();
                    state.play_selected_macros(&selected, 1, interval);
                }
            },
            "play_multiple" => {
                let selected = state.get_selected_macros();
                if !selected.is_empty() && !state.recorder.is_recording() {
                    let repeat = state.get_repeat_count();
                    let interval = state.get_macro_interval_ms();
                    state.play_selected_macros(&selected, repeat, interval);
                }
            },
            "select_all_macros" | "deselect_all_macros" | "toggle_help" => {
                // 需要UI状态，跳过
                debug!("快捷键 '{shortcut_name}' 需要UI状态访问，暂时跳过");
            },
            _ => {
                debug!("未知快捷键: {shortcut_name}");
            },
        }
    }
}
