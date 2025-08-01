use anyhow::Result;
use autopilot::mouse;
use log::debug;
use parking_lot::{Mutex, RwLock};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
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
        if self.current_macro_total_time == 0 {
            return 0.0;
        }
        let progress = current_duration as f32 / self.current_macro_total_time as f32 * 100.0;
        progress.clamp(0.0, 100.0)
    }
}

#[derive(Default, Clone)]
pub struct MacroPlayer {
    macros: Arc<Vec<Arc<SavedMacro>>>,
    is_playing: Arc<AtomicBool>,
    play_handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    interval_ms: u64,
    playback_status: Arc<RwLock<Arc<PlaybackStatus>>>,
}

impl MacroPlayer {
    pub fn new(macros: Vec<Arc<SavedMacro>>, interval_ms: u64) -> Self {
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
            // handle.abort();
        }
        // 更新状态为停止
        *self.playback_status.write() = PlaybackStatus::new_arc();
    }

    pub fn start_playing(&self, repeat_count: u32) {
        if self.is_playing.load(Ordering::Relaxed) {
            return;
        }

        let player = self.clone();
        let handle = thread::spawn(move || {
            if let Err(e) = player.play_async_with_repeat(repeat_count) {
                debug!("Error playing multi-macro: {e}");
            }
        });

        *self.play_handle.lock() = Some(handle);
    }

    fn play_async_with_repeat(&self, repeat_count: u32) -> Result<()> {
        self.is_playing.store(true, Ordering::Relaxed);

        let mut status = PlaybackStatus {
            is_playing: true,
            total_repeats: repeat_count,
            total_macros: self.macros.len(),
            ..Default::default()
        };

        for repeat in 1..=repeat_count {
            status.current_repeat = repeat;
            *self.playback_status.write() = Arc::new(status.clone());

            for (macro_index, saved_macro) in self.macros.iter().enumerate() {
                if !self.is_playing.load(Ordering::Relaxed) {
                    break;
                }
                let total_time = saved_macro.events.last().map(|e| e.timestamp).unwrap_or(0);
                let total_delay = saved_macro
                    .events
                    .iter()
                    .map(|e| {
                        if let MacroEventType::Delay { duration_ms } = e.event_type {
                            duration_ms
                        } else {
                            0
                        }
                    })
                    .sum::<u64>() as u128;
                let total_time = total_time + total_delay;

                status.current_macro_index = macro_index;
                status.current_macro_name = saved_macro.name.clone();
                status.current_macro_start_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                status.current_macro_total_time = total_time;
                *self.playback_status.write() = Arc::new(status.clone());

                if let Err(e) = self.play_macro_async(saved_macro) {
                    debug!("Error playing macro {}: {e}", saved_macro.name);
                }

                // 宏之间的间隔
                if macro_index < self.macros.len() - 1
                    && self.interval_ms > 0
                    && !self.sleep_efficient(self.interval_ms)
                {
                    break;
                }
            }
        }

        self.is_playing.store(false, Ordering::Relaxed);
        *self.playback_status.write() = PlaybackStatus::new_arc();

        Ok(())
    }

    fn play_macro_async(&self, saved_macro: &SavedMacro) -> Result<()> {
        let mut last_timestamp = 0u128;

        for event in &saved_macro.events {
            if !self.is_playing.load(Ordering::Relaxed) {
                break;
            }
            // 计算延时
            let delay = event.timestamp.saturating_sub(last_timestamp);
            if !self.sleep_efficient(delay as u64) {
                break;
            }

            // 执行事件
            match &event.event_type {
                MacroEventType::MouseMove { x, y } => {
                    let _ = mouse::move_to(autopilot::geometry::Point::new(*x as f64, *y as f64));
                },
                MacroEventType::MouseClick { button, pressed } => {
                    let button = match button {
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
                    if !self.sleep_efficient(*duration_ms) {
                        break;
                    }
                },
            }

            last_timestamp = event.timestamp;
        }

        Ok(())
    }

    #[inline]
    fn sleep_efficient(&self, delay_ms: u64) -> bool {
        if delay_ms == 0 {
            return true;
        }

        let start = Instant::now();
        let target_duration = Duration::from_millis(delay_ms);
        let sleep_time = match delay_ms {
            d if d < 1000 => d,
            _ => 1000,
        };

        let mut elapsed = Duration::from_millis(0);
        while elapsed < target_duration {
            let sleep_time = sleep_time.min((target_duration - elapsed).as_millis() as u64);
            thread::sleep(Duration::from_millis(sleep_time));
            if !self.is_playing.load(Ordering::Relaxed) {
                return false;
            }
            elapsed = start.elapsed();
        }

        true
    }
}
