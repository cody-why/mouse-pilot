use std::{collections::BTreeSet, sync::Arc};

use eframe::egui;
use parking_lot::{Mutex, RwLock};

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
    pub selected_macros: RwLock<BTreeSet<String>>,
    pub macro_interval_ms: Mutex<u64>,
    pub shortcuts: Arc<Vec<Shortcut>>,
    pub ui_context: egui::Context,
    pub mouse_position: Mutex<(i32, i32)>,
}

impl AppState {
    pub fn new(ctx: &egui::Context) -> Self {
        let shortcuts = Self::init_shortcuts();
        Self {
            player: Mutex::new(MacroPlayer::default()),
            macro_manager: MacroManager::new(),
            recorder: MacroRecorder::new(shortcuts.clone()),
            repeat_count: Mutex::new(1),
            selected_macros: Default::default(),
            macro_interval_ms: Mutex::new(0),
            shortcuts,
            ui_context: ctx.clone(),
            mouse_position: Mutex::new((0, 0)),
        }
    }

    fn init_shortcuts() -> Arc<Vec<Shortcut>> {
        // 初始化快捷键
        let shortcuts = vec![
            Shortcut::new("start_recording", egui::Key::F5, false, false, false, "开始录制", false),
            Shortcut::new("stop", egui::Key::F4, false, false, false, "停止录制/播放", false),
            Shortcut::new("play_once", egui::Key::F7, false, false, false, "播放一次", false),
            Shortcut::new("play_multiple", egui::Key::F8, false, false, false, "播放多次", false),
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
            Shortcut::new("help", egui::Key::F1, false, false, false, "显示/隐藏帮助", true),
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

    pub fn get_selected_macros(&self) -> BTreeSet<String> {
        self.selected_macros.read().clone()
    }

    pub fn set_selected_macros(&self, v: BTreeSet<String>) {
        *self.selected_macros.write() = v;
    }

    pub fn get_selected_count(&self) -> usize {
        self.selected_macros.read().len()
    }

    pub fn is_selected(&self, v: &str) -> bool {
        self.selected_macros.read().contains(v)
    }

    pub fn add_selected_macros(&self, v: &str) {
        self.selected_macros.write().insert(v.to_string());
    }

    pub fn remove_selected_macros(&self, v: &str) {
        self.selected_macros.write().remove(v);
    }

    pub fn clear_selected_macros(&self) {
        self.selected_macros.write().clear();
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

    pub fn play_selected_macros(&self, repeat_count: u32) {
        let selected_macros = self.get_selected_macros();
        let macro_interval_ms = self.get_macro_interval_ms();

        if selected_macros.is_empty() {
            return;
        }

        self._play_selected_macros(
            &selected_macros.into_iter().collect::<Vec<_>>(),
            repeat_count,
            macro_interval_ms,
        );
        self.ui_repaint_after_secs(0.2);
    }

    fn _play_selected_macros(
        &self, selected_macros: &[String], repeat_count: u32, macro_interval_ms: u64,
    ) {
        let macros_to_play = self.macro_manager.get_macros(selected_macros);

        if macros_to_play.is_empty() {
            return;
        }

        let player = MacroPlayer::new(macros_to_play, macro_interval_ms);
        player.start_playing(repeat_count);

        self.set_player(player);
    }

    pub fn ui_repaint_after_secs(&self, secs: f32) {
        self.ui_context.request_repaint_after_secs(secs);
    }

    pub fn get_mouse_position(&self) -> (i32, i32) {
        *self.mouse_position.lock()
    }

    pub fn set_mouse_position(&self, position: (i32, i32)) {
        *self.mouse_position.lock() = position;
    }
}
