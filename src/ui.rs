use eframe::egui;
use log::debug;
use std::collections::BTreeSet;
use std::sync::Arc;

use crate::hotkey::*;
use crate::state::AppState;

pub struct App {
    state: Arc<AppState>,
    ui_has_focus: bool,
    editing_macro_name: Option<String>,
    new_macro_name: String,
    deleting_macro: Option<String>,
    show_shortcuts_help: bool,
    // 全局快捷键相关
    global_listener: Option<GlobalHotkeyListener>,
    // 延时宏相关
    delay_macro_ms: u64,
    delay_macro_name: String,
}

impl App {
    pub fn new(ctx: &egui::Context) -> Self {
        let state = Arc::new(AppState::new(ctx));

        // 创建全局快捷键监听器
        let global_listener = GlobalHotkeyListener::new();

        let app = Self {
            state: state.clone(),
            ui_has_focus: false,
            editing_macro_name: None,
            new_macro_name: String::new(),
            deleting_macro: None,
            show_shortcuts_help: false,
            global_listener: Some(global_listener),
            delay_macro_ms: 1000,
            delay_macro_name: String::from("延时宏"),
        };

        // 启动全局快捷键监听
        if let Some(listener) = &app.global_listener {
            listener.start(state);
        }

        app
    }

    // 执行快捷键动作 - 保留用于UI内快捷键
    fn execute_shortcut(&mut self, shortcut_name: &str) {
        debug!("执行UI内快捷键: {shortcut_name}");
        match shortcut_name {
            "select_all_macros" => {
                let all_macros: BTreeSet<String> = self
                    .state
                    .macro_manager
                    .get_all_macros()
                    .iter()
                    .map(|m| m.name.clone())
                    .collect();
                self.state.set_selected_macros(all_macros);
            },
            "deselect_all_macros" => {
                self.state.clear_selected_macros();
            },
            "help" => {
                self.show_shortcuts_help = !self.show_shortcuts_help;
            },
            _ => {},
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui_has_focus = ctx.input(|i| i.focused);

        // UI 内快捷键
        if self.ui_has_focus {
            let mut shortcut_to_execute = None;
            ctx.input(|i| {
                for shortcut in self.state.shortcuts.iter() {
                    if i.key_pressed(shortcut.key) && shortcut.matches(shortcut.key, &i.modifiers) {
                        shortcut_to_execute = Some(shortcut.name.clone());
                    }
                }
            });

            if let Some(name) = shortcut_to_execute {
                self.execute_shortcut(&name);
            }
        }
        if self.state.recorder.is_recording() || self.state.is_playing() {
            // 强制刷新UI
            ctx.request_repaint_after_secs(0.2);
        }

        // 状态信息区域始终在底部，且要在所有面板之前调用
        self.render_status_panel(ctx);

        egui::SidePanel::left("macro_list")
            .resizable(true)
            .default_width(200.0)
            .min_width(150.0)
            .show(ctx, |ui| {
                self.render_macro_list(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_main_panel(ui);
        });

        // 删除确认对话框
        if let Some(macro_name) = &self.deleting_macro {
            match self.render_confirm_panel(
                ctx,
                "确认删除",
                &format!("确定要删除<{macro_name}>吗？"),
            ) {
                Some(true) => {
                    if let Err(e) = self.state.macro_manager.delete_macro(macro_name) {
                        debug!("Failed to delete macro: {e}");
                    }
                    // 删除后同步移除选中状态
                    self.state.clear_selected_macros();
                    self.deleting_macro = None;
                },
                Some(false) => {
                    self.deleting_macro = None;
                },
                None => {},
            }
        }

        // 快捷键帮助窗口
        if self.show_shortcuts_help {
            self.render_help_panel(ctx);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.state.recorder.stop_recording();

        // 停止全局快捷键监听
        if let Some(listener) = &self.global_listener {
            listener.stop();
        }
    }
}

impl App {
    /// 删除确认面板
    fn render_confirm_panel(
        &self, ctx: &egui::Context, title: &str, message: &str,
    ) -> Option<bool> {
        let mut result = None;
        egui::Window::new(title).collapsible(false).resizable(false).show(ctx, |ui| {
            ui.label(message);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌ 取消").clicked() {
                        result = Some(false);
                    }
                    ui.add_space(10.0);
                    if ui.button("✅ 确定").clicked() {
                        result = Some(true);
                    }
                });
            });
        });
        result
    }

    fn render_macro_list(&mut self, ui: &mut egui::Ui) {
        ui.label("宏列表");
        ui.separator();

        // 宏列表
        egui::ScrollArea::vertical().show(ui, |ui| {
            let macros = self.state.macro_manager.get_all_macros();
            // macros.sort_by(|a, b| a.name.cmp(&b.name));

            for macro_data in macros.iter() {
                ui.horizontal(|ui| {
                    let mut is_selected = self.state.is_selected(&macro_data.name);

                    if ui.checkbox(&mut is_selected, "").clicked() {
                        if is_selected {
                            self.state.add_selected_macros(&macro_data.name);
                        } else {
                            self.state.remove_selected_macros(&macro_data.name);
                        }
                    }

                    ui.label(&macro_data.name);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("📝").clicked() {
                            self.editing_macro_name = Some(macro_data.name.clone());
                            self.new_macro_name = macro_data.name.clone();
                        }

                        if ui.button("🗑").clicked() {
                            self.deleting_macro = Some(macro_data.name.clone());
                        }
                    });
                });

                // 重命名编辑框
                if let Some(editing_name) = &self.editing_macro_name {
                    if editing_name == &macro_data.name {
                        let old_name = editing_name.clone();
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.new_macro_name)
                                    .desired_width(ui.available_width() - 67.0),
                            );
                            if ui.button("✅").clicked() {
                                let new_name = self.new_macro_name.clone();
                                if !new_name.is_empty() && new_name != old_name {
                                    if let Err(e) =
                                        self.state.macro_manager.rename_macro(&old_name, &new_name)
                                    {
                                        debug!("Failed to rename macro: {e}");
                                    }
                                }
                                self.editing_macro_name = None;
                            }
                            if ui.button("❌").clicked() {
                                self.editing_macro_name = None;
                            }
                        });
                    }
                }
            }
        });
    }

    fn render_main_panel(&mut self, ui: &mut egui::Ui) {
        let is_recording = self.state.recorder.is_recording();
        let is_playing = self.state.is_playing();

        // 录制控制区域
        ui.group(|ui| {
            ui.separator();
            ui.label("录制控制");
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        !is_playing,
                        egui::Button::new(if is_recording {
                            "⏹ 停止录制 (F4)"
                        } else {
                            "🔴 开始录制 (F5)"
                        }),
                    )
                    .clicked()
                {
                    if is_recording {
                        self.state.recorder.stop_recording();
                    } else {
                        self.state.stop_player();
                        if let Err(e) = self.state.recorder.start_recording() {
                            debug!("Failed to start recording: {e}");
                        }
                    }
                }
            });

            // 手动录制控制
            if is_recording {
                ui.label("手动录制控制");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("添加鼠标点击").clicked() {
                        // self.state.recorder.add_mouse_click("Left", true, 100, 100);
                        // self.state.recorder.add_mouse_click("Left", false, 100, 100);
                    }

                    if ui.button("添加按键事件").clicked() {
                        // self.state.recorder.add_key_event("KeyA", true);
                        // self.state.recorder.add_key_event("KeyA", false);
                    }
                });

                // ui.horizontal(|ui| {
                //     if ui.button("添加图像识别事件").clicked() {
                //         self.state.recorder.add_image_find("screenshot.png", 0.8, 5000);
                //     }
                // });

                // ui.horizontal(|ui| {
                //     if ui.button("添加延时事件").clicked() {
                //         self.state.recorder.add_delay(1000);
                //     }
                // });
                ui.label("💡 提示: 手动添加未实现");
            }

            // 保存当前录制
            if self.state.recorder.get_event_count() > 0 {
                ui.separator();
                ui.label("保存录制");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_macro_name).desired_width(160.0),
                    );
                    if ui.button("💾 保存").clicked() && !self.new_macro_name.is_empty() {
                        let events = self.state.recorder.get_events();
                        if let Err(e) =
                            self.state.macro_manager.save_macro(&self.new_macro_name, events)
                        {
                            debug!("Failed to save macro: {e}");
                        } else {
                            self.state.recorder.clear_events();
                            self.new_macro_name.clear();
                        }
                    }
                });
            }
        });

        // 新增：添加延时宏区域
        ui.group(|ui| {
            ui.separator();
            ui.label("添加延时");

            ui.horizontal(|ui| {
                ui.label("延时:");
                ui.spacing_mut().item_spacing.x = 0.0;

                ui.add(egui::DragValue::new(&mut self.delay_macro_ms).speed(1000).suffix("ms"));
                // ▽▼
                if ui.add(egui::Button::new("▼").frame(false)).clicked() {
                    self.delay_macro_ms = self.delay_macro_ms.saturating_sub(1000);
                }
                // △▲
                if ui.add(egui::Button::new("▲").frame(false)).clicked() {
                    self.delay_macro_ms += 1000;
                }
            });
            ui.horizontal(|ui| {
                ui.label("名字:");
                // let text_edit_width = ui.available_width() - 150.0; // 预留按钮宽度
                ui.add(egui::TextEdit::singleline(&mut self.delay_macro_name).desired_width(50.0));

                if ui.button("➕ 添加").clicked() && !self.delay_macro_name.trim().is_empty() {
                    let macro_name = format!(
                        "{}({}s)",
                        self.delay_macro_name,
                        self.delay_macro_ms as f64 / 1000.0
                    );
                    let event = crate::event::MacroEvent {
                        event_type: crate::event::MacroEventType::Delay {
                            duration_ms: self.delay_macro_ms,
                        },
                        timestamp: 0,
                    };
                    if let Err(e) = self.state.macro_manager.save_macro(&macro_name, vec![event]) {
                        debug!("Failed to save delay macro: {e}");
                    }
                }
            });
        });

        let selected_count = self.state.get_selected_count();
        ui.group(|ui| {
            ui.separator();
            ui.label(format!("已选择: {selected_count}"));
            // 全选按钮, 清空按钮
            ui.horizontal(|ui| {
                if ui.button("全选").clicked() {
                    self.state.set_selected_macros(
                        self.state
                            .macro_manager
                            .get_all_macros()
                            .iter()
                            .map(|m| m.name.clone())
                            .collect(),
                    );
                }
                if ui.button("清空").clicked() {
                    self.state.clear_selected_macros();
                }
            });
        });

        // 播放控制区域
        ui.group(|ui| {
            ui.separator();
            ui.label("播放控制");
            ui.add_enabled_ui(!is_recording, |ui| {
                if selected_count > 0 || is_playing {
                    // 宏间隔设置
                    ui.horizontal(|ui| {
                        ui.label("宏间隔:");
                        ui.spacing_mut().item_spacing.x = 0.0;

                        let mut interval = self.state.get_macro_interval_ms();

                        if ui
                            .add(egui::DragValue::new(&mut interval).speed(1000).suffix("ms"))
                            .changed()
                        {
                            self.state.set_macro_interval_ms(interval);
                        }
                        if ui.add(egui::Button::new("▼").frame(false)).clicked() {
                            interval = interval.saturating_sub(1000);
                            self.state.set_macro_interval_ms(interval);
                        }
                        if ui.add(egui::Button::new("▲").frame(false)).clicked() {
                            interval = interval.saturating_add(1000);
                            self.state.set_macro_interval_ms(interval);
                        }
                    });
                    ui.horizontal(|ui| {
                        // 播放一次
                        if ui
                            .button(if is_playing {
                                "⏹ 停止播放 (F4)"
                            } else {
                                "▶ 播放 1 次 (F7)"
                            })
                            .clicked()
                        {
                            if is_playing {
                                self.state.stop_player();
                            } else {
                                self.play_selected_macros(1);
                            }
                        }

                        // 播放多次
                        if ui
                            .button(if is_playing {
                                "⏹ 停止播放 (F4)"
                            } else {
                                "▶ 播放 (F8)"
                            })
                            .clicked()
                        {
                            if is_playing {
                                self.state.stop_player();
                            } else {
                                // 多次播放选中的宏
                                self.play_selected_macros(self.state.get_repeat_count());
                            }
                        }

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;

                            // 播放次数
                            let mut repeat_count = self.state.get_repeat_count();

                            if ui
                                .add(egui::DragValue::new(&mut repeat_count).speed(1).suffix("次"))
                                .changed()
                            {
                                self.state.set_repeat_count(repeat_count);
                            }
                            if ui.add(egui::Button::new("▼").frame(false)).clicked() {
                                repeat_count = repeat_count.saturating_sub(1);
                                self.state.set_repeat_count(repeat_count);
                            }
                            if ui.add(egui::Button::new("▲").frame(false)).clicked() {
                                repeat_count = repeat_count.saturating_add(1);
                                self.state.set_repeat_count(repeat_count);
                            }
                        });
                    });
                } else {
                    ui.label("请先选择要播放的宏");
                }
            });
        });
    }

    // 状态信息区域
    fn render_status_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let total_macros = self.state.macro_manager.get_macro_count();
                ui.label(format!("宏数量: {total_macros}"));

                // 录制状态
                if self.state.recorder.is_recording() {
                    ui.separator();
                    let time_elapsed = self.state.recorder.get_time_elapsed();
                    let click_time_elapsed = self.state.recorder.get_click_time_elapsed();
                    let events_count = self.state.recorder.get_event_count();
                    ui.label(format!(
                        "🔴 录制中: {:.1}s | 点击: {:.1}s | 事件: {events_count}",
                        time_elapsed as f64 / 1000.0,
                        click_time_elapsed as f64 / 1000.0
                    ));
                }

                ui.separator();

                // 播放状态
                let status_text = if self.state.is_playing() {
                    let playback_status = self.state.get_player_playback_status();
                    let progress = playback_status.get_progress();
                    let mut s = format!("▶ {progress:.1}%");
                    if playback_status.total_repeats > 1 {
                        s += &format!(
                            " | 第 {}/{} 次",
                            playback_status.current_repeat, playback_status.total_repeats
                        );
                    }

                    if !playback_status.current_macro_name.is_empty() {
                        s += &format!(
                            " | {} | {}/{}",
                            playback_status.current_macro_name,
                            playback_status.current_macro_index + 1,
                            playback_status.total_macros
                        );
                    }

                    s
                } else {
                    String::from("⏹ 未播放")
                };
                ui.label(status_text);

                ui.separator();

                // 显示快捷键提示
                ui.label("💡 F1 查看帮助");
            });
        });
    }

    fn render_help_panel(&mut self, ctx: &egui::Context) {
        egui::Window::new("快捷键帮助")
            .collapsible(true)
            .resizable(true)
            .default_size([300.0, 300.0])
            // 居中显示
            .anchor(egui::Align2::CENTER_TOP, [0.0, 0.0])
            // .default_pos([(ctx.screen_rect().width() - 300.0) / 2.0, 0.0])
            .show(ctx, |ui| {
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for shortcut in self.state.shortcuts.iter() {
                        ui.horizontal(|ui| {
                            ui.label(&shortcut.description);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let color = if ui.visuals().dark_mode {
                                        egui::Color32::LIGHT_BLUE
                                    } else {
                                        egui::Color32::BLUE
                                    };
                                    ui.colored_label(color, shortcut.display_text());
                                },
                            );
                        });
                    }
                });
            });
    }

    fn play_selected_macros(&mut self, repeat_count: u32) {
        self.state.play_selected_macros(repeat_count);
    }
}
