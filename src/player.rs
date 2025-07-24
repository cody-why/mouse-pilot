use anyhow::Result;
use autopilot::mouse;
use log::debug;
use parking_lot::Mutex;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

use crate::{event::*, macro_manager::SavedMacro};

// 新增：支持顺序播放多个宏的播放器
#[derive(Default)]
pub struct MacroPlayer {
    macros: Vec<SavedMacro>,
    is_playing: Arc<AtomicBool>,
    play_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    interval_ms: u64,
}

impl MacroPlayer {
    pub fn new(macros: Vec<SavedMacro>, interval_ms: u64) -> Self {
        Self {
            macros,
            is_playing: Arc::new(AtomicBool::new(false)),
            play_handle: Arc::new(Mutex::new(None)),
            interval_ms,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        if !self.is_playing.load(Ordering::SeqCst) {
            return;
        }
        self.is_playing.store(false, Ordering::SeqCst);

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

        for repeat_index in 0..repeat_count {
            if !self.is_playing.load(Ordering::SeqCst) {
                break;
            }

            debug!("开始第 {} 次播放", repeat_index + 1);

            for (i, macro_) in self.macros.iter().enumerate() {
                if !self.is_playing.load(Ordering::SeqCst) {
                    break;
                }

                debug!("Playing macro: {}", macro_.name);

                let mut last_timestamp = 0;
                for event in &macro_.events {
                    if !self.is_playing.load(Ordering::SeqCst) {
                        break;
                    }

                    let delay = event.timestamp - last_timestamp;
                    std::thread::sleep(Duration::from_millis(delay as u64));
                    last_timestamp = event.timestamp;

                    match &event.event_type {
                        MacroEventType::MouseMove { x, y } => {
                            let _ = mouse::move_to(autopilot::geometry::Point::new(
                                *x as f64, *y as f64,
                            ));
                        },
                        MacroEventType::MouseClick {
                            button,
                            pressed,
                            // x: _,
                            // y: _,
                        } => {
                            // debug!("Mouse click: {button:?} {pressed}");

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
                            std::thread::sleep(Duration::from_millis(*duration_ms));
                        },
                    }
                }

                // 在宏之间添加间隔（除了最后一个宏）
                if i < self.macros.len() - 1 && self.interval_ms > 0 {
                    if !self.is_playing.load(Ordering::SeqCst) {
                        break;
                    }
                    debug!("Waiting {}ms before next macro...", self.interval_ms);
                    std::thread::sleep(Duration::from_millis(self.interval_ms));
                }
            }
        }

        self.is_playing.store(false, Ordering::SeqCst);
        Ok(())
    }
}

impl Clone for MacroPlayer {
    fn clone(&self) -> Self {
        Self {
            macros: self.macros.clone(),
            is_playing: self.is_playing.clone(),
            play_handle: Arc::new(Mutex::new(None)),
            interval_ms: self.interval_ms,
        }
    }
}
