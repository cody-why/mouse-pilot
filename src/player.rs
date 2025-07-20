use anyhow::Result;
use autopilot::mouse;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use crate::event::*;

pub struct MacroPlayer {
    events: Vec<MacroEvent>,
    is_playing: Arc<AtomicBool>,
    play_handle: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
}

impl MacroPlayer {
    pub fn new(events: Vec<MacroEvent>) -> Self {
        Self {
            events,
            is_playing: Arc::new(AtomicBool::new(false)),
            play_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        self.is_playing.store(false, Ordering::SeqCst);

        // 清理播放线程句柄
        if let Some(handle) = self.play_handle.lock().unwrap().take() {
            // std::thread::JoinHandle 没有 abort 方法，只能等待线程自然结束
            let _ = handle.join();
        }
    }

    pub fn start_playing(&self, repeat_count: u32) {
        if self.is_playing.load(Ordering::SeqCst) {
            return;
        }

        let player = self.clone();
        let handle = std::thread::spawn(move || {
            if let Err(e) = player.play_sync(repeat_count) {
                eprintln!("Error playing macro: {e}");
            }
        });

        *self.play_handle.lock().unwrap() = Some(handle);
    }

    fn play_sync(&self, repeat_count: u32) -> Result<()> {
        self.is_playing.store(true, Ordering::SeqCst);

        for _ in 0..repeat_count {
            if !self.is_playing.load(Ordering::SeqCst) {
                break;
            }

            let mut last_timestamp = 0;
            for event in &self.events {
                if !self.is_playing.load(Ordering::SeqCst) {
                    break;
                }

                let delay = event.timestamp - last_timestamp;
                std::thread::sleep(Duration::from_millis(delay as u64));
                last_timestamp = event.timestamp;

                match &event.event_type {
                    MacroEventType::MouseMove { x, y } => {
                        // 使用 autopilot 进行鼠标移动
                        let _ =
                            mouse::move_to(autopilot::geometry::Point::new(*x as f64, *y as f64));
                    },
                    MacroEventType::MouseClick {
                        button: _,
                        pressed,
                        x: _,
                        y: _,
                    } => {
                        // 使用 autopilot 进行鼠标点击
                        if *pressed {
                            mouse::toggle(mouse::Button::Left, true);
                        } else {
                            mouse::toggle(mouse::Button::Left, false);
                        }
                    },
                    MacroEventType::KeyPress { key } => {
                        // 使用 autopilot 进行按键
                        eprintln!("Key press: {key}");
                    },
                    MacroEventType::KeyRelease { key } => {
                        eprintln!("Key release: {key}");
                    },
                    MacroEventType::ImageFind {
                        image_path,
                        confidence: _,
                        timeout: _,
                    } => {
                        // 简化的图像识别功能
                        eprintln!("Looking for image: {image_path}");
                        // 这里可以添加实际的图像识别逻辑
                    },
                    MacroEventType::WaitForImage {
                        image_path,
                        confidence: _,
                        timeout,
                    } => {
                        // 简化的等待图像功能
                        eprintln!("Waiting for image: {image_path} (timeout: {timeout}ms)");
                        std::thread::sleep(Duration::from_millis(*timeout));
                    },
                    MacroEventType::Screenshot { path } => {
                        // 简化的截图功能
                        eprintln!("Taking screenshot: {path}");
                        // 这里可以添加实际的截图逻辑
                    },
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
            events: self.events.clone(),
            is_playing: self.is_playing.clone(),
            play_handle: Arc::new(Mutex::new(None)),
        }
    }
}
