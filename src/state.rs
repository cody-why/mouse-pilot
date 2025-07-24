use parking_lot::Mutex;

use crate::{macro_manager::MacroManager, player::MacroPlayer, recorder::MacroRecorder};

pub struct AppState {
    pub player: Mutex<MacroPlayer>,
    pub macro_manager: MacroManager,
    pub recorder: MacroRecorder,
    pub repeat_count: Mutex<u32>,
    pub selected_macros: Mutex<Vec<String>>,
    pub macro_interval_ms: Mutex<u64>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            player: Mutex::new(MacroPlayer::default()),
            macro_manager: MacroManager::new(),
            recorder: MacroRecorder::new(),
            repeat_count: Mutex::new(1),
            selected_macros: Mutex::new(Vec::new()),
            macro_interval_ms: Mutex::new(0),
        }
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
