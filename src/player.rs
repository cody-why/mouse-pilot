use anyhow::Result;
use autopilot::mouse;
use log::debug;
use parking_lot::{Mutex, RwLock};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

use crate::{event::*, macro_manager::SavedMacro};

// 播放进度信息
#[derive(Debug, Clone, Default)]
pub struct PlaybackStatus {
    pub is_playing: bool,
    pub current_repeat: u32,
    pub total_repeats: u32,
    pub current_macro_index: usize,
    pub total_macros: usize,
    pub current_macro_name: String,
    pub current_macro_start_time: u128, // 当前宏开始播放的UNIX时间戳(ms)
    pub current_macro_total_time: u128, // 当前宏总时长(ms)
}

impl PlaybackStatus {
    pub fn new_arc() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn get_progress(&self) -> f32 {
        let current_duration = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            - self.current_macro_start_time;
        current_duration as f32 / self.current_macro_total_time as f32 * 100.0
    }
}

#[derive(Default, Clone)]
pub struct MacroPlayer {
    macros: Arc<Vec<SavedMacro>>,
    is_playing: Arc<AtomicBool>,
    play_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    interval_ms: u64,
    playback_status: Arc<RwLock<Arc<PlaybackStatus>>>,
}

impl MacroPlayer {
    pub fn new(macros: Vec<SavedMacro>, interval_ms: u64) -> Self {
        Self {
            macros: Arc::new(macros),
            is_playing: Arc::new(AtomicBool::new(false)),
            play_handle: Arc::new(Mutex::new(None)),
            interval_ms,
            playback_status: Arc::new(RwLock::new(PlaybackStatus::new_arc())),
        }
    }

    pub fn get_playback_status(&self) -> Arc<PlaybackStatus> {
        self.playback_status.read().clone()
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        if !self.is_playing.load(Ordering::SeqCst) {
            return;
        }
        self.is_playing.store(false, Ordering::SeqCst);

        // 更新状态为停止
        *self.playback_status.write() = PlaybackStatus::new_arc();

        if let Some(handle) = self.play_handle.lock().take() {
            let _ = handle.join();
        }
    }

    pub fn start_playing(&self) {
        self.start_playing_with_repeat(1);
    }

    pub fn start_playing_with_repeat(&self, repeat_count: u32) {
        if self.is_playing.load(Ordering::SeqCst) {
            return;
        }

        let player = self.clone();
        let handle = std::thread::spawn(move || {
            if let Err(e) = player.play_sync_with_repeat(repeat_count) {
                debug!("Error playing multi-macro: {e}");
            }
        });

        *self.play_handle.lock() = Some(handle);
    }

    fn play_sync_with_repeat(&self, repeat_count: u32) -> Result<()> {
        self.is_playing.store(true, Ordering::SeqCst);

        let total_macros = self.macros.len();

        for repeat_index in 0..repeat_count {
            if !self.is_playing.load(Ordering::SeqCst) {
                break;
            }

            debug!("开始第 {} 次播放", repeat_index + 1);

            for (i, macro_) in self.macros.iter().enumerate() {
                if !self.is_playing.load(Ordering::SeqCst) {
                    break;
                }
                let total_duration = macro_.events.last().map(|e| e.timestamp).unwrap_or(0);
                let total_delay = macro_
                    .events
                    .iter()
                    .map(|e| match e.event_type {
                        MacroEventType::Delay { duration_ms } => duration_ms as u128,
                        _ => 0,
                    })
                    .sum::<u128>();
                // 更新播放状态
                let macro_start_timestamp = macro_.events.first().map(|e| e.timestamp).unwrap_or(0);
                let macro_end_timestamp = total_duration + total_delay;
                let start_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let mut last_timestamp = macro_start_timestamp;
                for event in &macro_.events {
                    let delay = event.timestamp - last_timestamp;
                    self.sleep_with_interval(delay as u64);
                    if !self.is_playing.load(Ordering::SeqCst) {
                        break;
                    }
                    last_timestamp = event.timestamp;

                    let status = PlaybackStatus {
                        is_playing: true,
                        current_repeat: repeat_index + 1,
                        total_repeats: repeat_count,
                        current_macro_index: i,
                        total_macros,
                        current_macro_name: macro_.name.clone(),
                        current_macro_start_time: start_time,
                        current_macro_total_time: macro_end_timestamp - macro_start_timestamp,
                    };
                    *self.playback_status.write() = Arc::new(status);

                    // 事件执行
                    match &event.event_type {
                        MacroEventType::MouseMove { x, y } => {
                            let _ = mouse::move_to(autopilot::geometry::Point::new(
                                *x as f64, *y as f64,
                            ));
                        },
                        MacroEventType::MouseClick { button, pressed } => {
                            let button = match *button {
                                Button::Left => mouse::Button::Left,
                                Button::Right => mouse::Button::Right,
                                Button::Middle => mouse::Button::Middle,
                            };
                            mouse::toggle(button, *pressed);
                        },
                        MacroEventType::KeyPress { key } => {
                            debug!("Key press: {key}");
                        },
                        MacroEventType::KeyRelease { key } => {
                            debug!("Key release: {key}");
                        },
                        MacroEventType::ImageFind {
                            image_path,
                            confidence: _,
                            timeout: _,
                        } => {
                            debug!("Looking for image: {image_path}");
                        },
                        MacroEventType::Delay { duration_ms } => {
                            debug!("Delay: {duration_ms}ms");
                            self.sleep_with_interval(*duration_ms);
                        },
                    }
                }

                // 在宏之间添加间隔（除了最后一个宏）
                if i < self.macros.len() - 1 && self.interval_ms > 0 {
                    if !self.is_playing.load(Ordering::SeqCst) {
                        break;
                    }
                    debug!("Waiting {}ms before next macro...", self.interval_ms);
                    self.sleep_with_interval(self.interval_ms);
                }
            }
        }

        self.is_playing.store(false, Ordering::SeqCst);
        // 播放完成，更新状态
        *self.playback_status.write() = PlaybackStatus::new_arc();
        Ok(())
    }

    #[inline]
    fn sleep_with_interval(&self, delay: u64) {
        let mut delay = delay;
        while delay > 0 {
            if !self.is_playing.load(Ordering::SeqCst) {
                break;
            }
            let interval = std::cmp::min(delay, 1000);
            std::thread::sleep(Duration::from_millis(interval));
            delay -= interval;
        }
    }
}
