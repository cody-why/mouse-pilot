use crate::{player::MacroPlayer, recorder::MacroRecorder};
use eframe::egui;
use std::sync::Arc;

pub struct App {
    recorder: Arc<MacroRecorder>,
    player: Option<Arc<MacroPlayer>>,
    repeat_count: u32,
    ui_has_focus: bool,
}

impl App {
    pub fn new() -> Self {
        let recorder = Arc::new(MacroRecorder::new());
        Self {
            recorder,
            player: None,
            repeat_count: 1,
            ui_has_focus: false,
        }
    }

    pub fn get_recorder(&self) -> Arc<MacroRecorder> {
        self.recorder.clone()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检测UI是否有焦点
        // let previous_focus = self.ui_has_focus;
        self.ui_has_focus = ctx.input(|i| i.focused);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("鼠标键盘宏录制器");

            let is_recording = self.recorder.is_recording();
            let is_playing = self.player.as_ref().is_some_and(|p| p.is_playing());

            // 录制控制区域
            ui.group(|ui| {
                ui.label("录制控制");
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            !is_playing,
                            egui::Button::new(if is_recording {
                                "⏹ 停止录制"
                            } else {
                                "🔴 开始录制"
                            }),
                        )
                        .clicked()
                    {
                        if is_recording {
                            self.recorder.stop_recording();
                            let events = self.recorder.get_events();
                            if !events.is_empty() {
                                self.player = Some(Arc::new(MacroPlayer::new(events)));
                            }
                        } else {
                            if let Err(e) = self.recorder.start_recording() {
                                eprintln!("Failed to start recording: {e}");
                            }
                            self.player = None;
                        }
                    }

                    if is_recording {
                        ui.label("🔴 录制中...");
                    }
                });

                // 手动录制控制
                if is_recording {
                    ui.separator();
                    ui.label("手动录制控制:");
                    ui.horizontal(|ui| {
                        if ui.button("添加鼠标点击").clicked() {
                            // 添加一个示例鼠标点击事件
                            self.recorder.add_mouse_click("Left", true, 100, 100);
                            self.recorder.add_mouse_click("Left", false, 100, 100);
                        }

                        if ui.button("添加按键事件").clicked() {
                            // 添加一个示例按键事件
                            self.recorder.add_key_event("KeyA", true);
                            self.recorder.add_key_event("KeyA", false);
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("添加图像识别事件").clicked() {
                            // 添加图像识别事件
                            self.recorder.add_image_find("screenshot.png", 0.8, 5000);
                        }

                        if ui.button("添加等待图像事件").clicked() {
                            // 添加等待图像事件
                            self.recorder.add_wait_for_image("button.png", 0.9, 10000);
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("添加截图事件").clicked() {
                            // 添加截图事件
                            self.recorder.add_screenshot("screenshot.png");
                        }
                    });

                    ui.label("💡 提示: 使用手动控制添加事件");
                }

                // 添加焦点状态显示
                if self.ui_has_focus {
                    ui.colored_label(egui::Color32::ORANGE, "⚠️ UI 有焦点 - 录制可能捕获 UI 事件");
                    ui.label("为了最佳效果, 在录制前点击应用窗口外");
                }

                // 添加功能说明
                ui.separator();
                ui.label("🔧 高级功能:");
                ui.label("• 鼠标和键盘事件录制");
                ui.label("• 图像识别和点击");
                ui.label("• 等待图像出现");
                ui.label("• 截图捕获");
            });

            // 播放控制区域
            ui.group(|ui| {
                ui.label("播放控制");
                ui.add_enabled_ui(!is_recording, |ui| {
                    if let Some(player) = &self.player {
                        ui.horizontal(|ui| {
                            // 播放一次
                            if ui
                                .button(if is_playing {
                                    "⏹ 停止播放"
                                } else {
                                    "▶ 播放 1 次"
                                })
                                .clicked()
                            {
                                if is_playing {
                                    player.stop();
                                } else {
                                    player.start_playing(1);
                                }
                            }

                            // 播放多次
                            if ui
                                .button(if is_playing {
                                    "⏹ 停止播放"
                                } else {
                                    "▶ 播放"
                                })
                                .clicked()
                            {
                                if is_playing {
                                    player.stop();
                                } else {
                                    player.start_playing(self.repeat_count);
                                }
                            }

                            ui.horizontal(|ui| {
                                // 播放次数
                                ui.add(
                                    egui::DragValue::new(&mut self.repeat_count)
                                        .speed(1)
                                        .range(1..=10000),
                                );
                                ui.label("次");
                            });

                            if is_playing {
                                ui.label("▶ 播放中...");
                            }
                        });
                    } else {
                        ui.label("没有可用的录制宏");
                    }
                });
            });

            // 状态信息区域
            ui.group(|ui| {
                ui.label("状态");
                let events_count = self.recorder.get_events_count();
                ui.label(format!("录制事件: {events_count}"));

                if let Some(_player) = &self.player {
                    ui.label("宏已准备好播放".to_string());
                }
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // 应用退出时清理监听线程
        self.recorder.stop_recording();
    }
}
