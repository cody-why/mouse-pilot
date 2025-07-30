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
    time::{Duration, Instant},
};

use crate::{event::*, key::*, macro_manager::SavedMacro};

// 播放进度信息
#[derive(Debug, Clone, Default)]
pub struct PlaybackStatus {
    pub is_playing: bool,
    pub current_repeat: u32,
    pub total_repeats: u32,
    pub current_macro_index: usize,
    pub total_macros: usize,
    pub current_macro_name: String,
    pub current_macro_start_time: u128, // 当前宏开始播放的时间戳(ms)
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
        let progress = current_duration as f32 / self.current_macro_total_time as f32 * 100.0;
        progress.clamp(0.0, 100.0)
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
        self.is_playing.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        if !self.is_playing.load(Ordering::Relaxed) {
            return;
        }
        self.is_playing.store(false, Ordering::Relaxed);

        if let Some(_handle) = self.play_handle.lock().take() {
            // let _ = handle.join();
        }
        // 更新状态为停止
        *self.playback_status.write() = PlaybackStatus::new_arc();
    }

    pub fn start_playing(&self) {
        self.start_playing_with_repeat(1);
    }

    pub fn start_playing_with_repeat(&self, repeat_count: u32) {
        if self.is_playing.load(Ordering::Relaxed) {
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
        self.is_playing.store(true, Ordering::Relaxed);

        let total_macros = self.macros.len();

        for repeat_index in 0..repeat_count {
            if !self.is_playing.load(Ordering::Relaxed) {
                break;
            }

            debug!("开始第 {} 次播放", repeat_index + 1);

            for (i, macro_) in self.macros.iter().enumerate() {
                if !self.is_playing.load(Ordering::Relaxed) {
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

                // 更新播放状态 - 只在宏开始时更新一次
                let macro_start_timestamp = macro_.events.first().map(|e| e.timestamp).unwrap_or(0);
                let macro_end_timestamp = total_duration + total_delay;
                let start_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();

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

                let mut last_timestamp = macro_start_timestamp;

                for event in &macro_.events {
                    let delay = event.timestamp - last_timestamp;
                    if !self.sleep_efficient(delay as u64) {
                        break;
                    }
                    last_timestamp = event.timestamp;

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
                        MacroEventType::KeyPress { key } => match pilot_key_code_from_str(key) {
                            KeyConvert::Keycode(key_code) => {
                                autopilot::key::toggle(&key_code, true, &[], 0);
                            },
                            KeyConvert::Character(key_code) => {
                                autopilot::key::toggle(&key_code, true, &[], 0);
                            },
                            _ => {
                                debug!("无法识别的按键: {key}");
                            },
                        },

                        MacroEventType::KeyRelease { key } => match pilot_key_code_from_str(key) {
                            KeyConvert::Keycode(key_code) => {
                                autopilot::key::toggle(&key_code, false, &[], 0);
                            },
                            KeyConvert::Character(key_code) => {
                                autopilot::key::toggle(&key_code, false, &[], 0);
                            },
                            _ => {
                                debug!("无法识别的按键: {key}");
                            },
                        },
                        MacroEventType::Delay { duration_ms } => {
                            debug!("Delay: {duration_ms}ms");
                            if !self.sleep_efficient(*duration_ms) {
                                break;
                            }
                        },
                    }
                }

                // 在宏之间添加间隔（除了最后一个宏）
                if i < self.macros.len() - 1 && self.interval_ms > 0 {
                    if !self.is_playing.load(Ordering::Relaxed) {
                        break;
                    }
                    debug!("Waiting {}ms before next macro...", self.interval_ms);
                    if !self.sleep_efficient(self.interval_ms) {
                        break;
                    }
                }
            }
        }

        self.is_playing.store(false, Ordering::Relaxed);
        // 播放完成，更新状态
        *self.playback_status.write() = PlaybackStatus::new_arc();
        Ok(())
    }

    /// 优化的睡眠方法，长延迟时，可以中断
    /// 返回false表示播放被中断
    #[inline]
    fn sleep_efficient(&self, delay_ms: u64) -> bool {
        if delay_ms == 0 {
            return true;
        }

        let start = Instant::now();
        let target_duration = Duration::from_millis(delay_ms);

        // 根据延迟时间调整检查间隔
        let check_interval = if delay_ms < 500 {
            Duration::from_millis(delay_ms)
        } else if delay_ms < 1000 {
            Duration::from_millis(500)
        } else {
            Duration::from_millis(1000)
        };

        while start.elapsed() < target_duration {
            let remaining = target_duration - start.elapsed();
            let sleep_duration = std::cmp::min(remaining, check_interval);
            std::thread::sleep(sleep_duration);

            if !self.is_playing.load(Ordering::Relaxed) {
                return false;
            }
        }

        true
    }
}
