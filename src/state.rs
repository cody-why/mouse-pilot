use std::sync::Arc;

use eframe::egui;
use parking_lot::Mutex;

use crate::{
    hotkey::Shortcut,
    macro_manager::MacroManager,
    player::{MacroPlayer, PlaybackStatus},
    recorder::MacroRecorder,
};

pub struct AppState {
    pub player: Mutex<MacroPlayer>,
    pub macro_manager: MacroManager,
    pub recorder: MacroRecorder,
    pub repeat_count: Mutex<u32>,
    pub selected_macros: Mutex<Vec<String>>,
    pub macro_interval_ms: Mutex<u64>,
    pub shortcuts: Arc<Vec<Shortcut>>,
}

impl AppState {
    pub fn new() -> Self {
        let shortcuts = Self::init_shortcuts();
        Self {
            player: Mutex::new(MacroPlayer::default()),
            macro_manager: MacroManager::new(),
            recorder: MacroRecorder::new(shortcuts.clone()),
            repeat_count: Mutex::new(1),
            selected_macros: Mutex::new(Vec::new()),
            macro_interval_ms: Mutex::new(0),
            shortcuts,
        }
    }

    fn init_shortcuts() -> Arc<Vec<Shortcut>> {
        // 初始化快捷键
        let shortcuts = vec![
            Shortcut::new("start_recording", egui::Key::F5, false, false, false, "开始录制", false),
            Shortcut::new("stop_recording", egui::Key::F4, false, false, false, "停止录制", false),
            Shortcut::new("play_once", egui::Key::F7, false, false, false, "播放一次", false),
            Shortcut::new("play_multiple", egui::Key::F8, false, false, false, "播放多次", false),
            Shortcut::new("stop_playback", egui::Key::F9, false, false, false, "停止播放", false),
            Shortcut::new(
                "clear_recording",
                egui::Key::Delete,
                true,
                false,
                false,
                "清空录制",
                false,
            ),
            Shortcut::new("select_all_macros", egui::Key::A, true, false, false, "全选宏", true),
            Shortcut::new(
                "deselect_all_macros",
                egui::Key::D,
                true,
                false,
                false,
                "取消全选",
                true,
            ),
            Shortcut::new("toggle_help", egui::Key::F1, false, false, false, "显示/隐藏帮助", true),
        ];
        Arc::new(shortcuts)
    }

    pub fn set_player(&self, player: MacroPlayer) {
        *self.player.lock() = player;
    }

    pub fn stop_player(&self) {
        self.player.lock().stop();
    }

    pub fn is_playing(&self) -> bool {
        self.player.lock().is_playing()
    }

    pub fn get_repeat_count(&self) -> u32 {
        *self.repeat_count.lock()
    }

    pub fn set_repeat_count(&self, v: u32) {
        *self.repeat_count.lock() = v;
    }

    pub fn get_selected_macros(&self) -> Vec<String> {
        self.selected_macros.lock().clone()
    }

    pub fn get_selected_count(&self) -> usize {
        self.selected_macros.lock().len()
    }

    pub fn set_selected_macros(&self, v: Vec<String>) {
        *self.selected_macros.lock() = v;
    }

    pub fn clear_selected_macros(&self) {
        self.selected_macros.lock().clear();
    }

    pub fn get_macro_interval_ms(&self) -> u64 {
        *self.macro_interval_ms.lock()
    }

    pub fn set_macro_interval_ms(&self, v: u64) {
        *self.macro_interval_ms.lock() = v;
    }

    pub fn get_player_playback_status(&self) -> Arc<PlaybackStatus> {
        self.player.lock().get_playback_status()
    }

    pub fn play_selected_macros(
        &self, selected_macros: &[String], repeat_count: u32, macro_interval_ms: u64,
    ) {
        // 停止当前播放
        self.stop_player();

        // 收集选中的宏事件
        let macros = self.macro_manager.get_macros(selected_macros);

        if !macros.is_empty() {
            // 创建多宏播放器
            let multi_player = MacroPlayer::new(macros, macro_interval_ms);
            multi_player.start_playing_with_repeat(repeat_count);
            self.set_player(multi_player);
        }
    }
}
