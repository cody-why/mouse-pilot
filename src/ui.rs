use eframe::egui;
use log::debug;
use std::sync::Arc;

use crate::hotkey::*;
use crate::state::AppState;

pub struct App {
    // recorder: Arc<MacroRecorder>,
    // player: Option<Arc<MultiMacroPlayer>>,
    // macro_manager: Arc<MacroManager>,
    state: Arc<AppState>,
    ui_has_focus: bool,
    editing_macro_name: Option<String>,
    new_macro_name: String,
    deleting_macro: Option<String>,
    // 快捷键相关
    shortcuts: Vec<Shortcut>,
    show_shortcuts_help: bool,
    // 全局快捷键相关
    global_listener: Option<GlobalHotkeyListener>,
    // 延时宏相关
    delay_macro_ms: u64,
    delay_macro_name: String,
}

impl App {
    pub fn new() -> Self {
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

        // 创建全局快捷键监听器
        let global_listener = GlobalHotkeyListener::new(shortcuts.clone());

        let state = Arc::new(AppState::new());

        let app = Self {
            state: state.clone(),
            ui_has_focus: false,
            editing_macro_name: None,
            new_macro_name: String::new(),
            deleting_macro: None,
            shortcuts,
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
                let all_macros: Vec<String> = self
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
            "toggle_help" => {
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
                for shortcut in &self.shortcuts {
                    if i.key_pressed(shortcut.key) && shortcut.matches(shortcut.key, &i.modifiers) {
                        shortcut_to_execute = Some(shortcut.name.clone());
                    }
                }
            });

            if let Some(name) = shortcut_to_execute {
                self.execute_shortcut(&name);
            }
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
            let mut macros = self.state.macro_manager.get_all_macros();
            macros.sort_by(|a, b| a.name.cmp(&b.name));

            let selected_macros = self.state.get_selected_macros();

            for macro_data in macros {
                ui.horizontal(|ui| {
                    let mut is_selected = selected_macros.contains(&macro_data.name);

                    if ui.checkbox(&mut is_selected, "").clicked() {
                        let mut new_selected = self.state.get_selected_macros();
                        if is_selected {
                            if !new_selected.contains(&macro_data.name) {
                                new_selected.push(macro_data.name.clone());
                            }
                        } else {
                            new_selected.retain(|name| name != &macro_data.name);
                        }
                        self.state.set_selected_macros(new_selected);
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

                if is_recording {
                    ui.label("🔴 录制中...");
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
            if !self.state.recorder.get_events().is_empty() {
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
                ui.label("延时(毫秒):");
                ui.add(
                    egui::DragValue::new(&mut self.delay_macro_ms)
                        .speed(100)
                        .range(1..=600_000)
                        .suffix("ms"),
                );
                ui.label("名字:");
                // 让输入框自适应剩余宽度
                let text_edit_width = ui.available_width() - 150.0; // 预留按钮宽度
                ui.add(
                    egui::TextEdit::singleline(&mut self.delay_macro_name)
                        .desired_width(text_edit_width.max(40.0)),
                );

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

        // 播放控制区域
        ui.group(|ui| {
            ui.separator();
            ui.label("播放控制");
            ui.add_enabled_ui(!is_recording, |ui| {
                if !self.state.get_selected_macros().is_empty() {
                    ui.horizontal(|ui| {
                        // 播放一次
                        if ui
                            .button(if is_playing {
                                "⏹ 停止播放 (F9)"
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
                                "⏹ 停止播放 (F9)"
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
                            // 播放次数
                            let mut repeat_count = self.state.get_repeat_count();
                            if ui
                                .add(
                                    egui::DragValue::new(&mut repeat_count)
                                        .speed(1)
                                        .range(1..=10000),
                                )
                                .changed()
                            {
                                self.state.set_repeat_count(repeat_count);
                            }
                            ui.label("次");
                        });
                    });
                    if is_playing {
                        ui.label("▶ 播放中...");
                    }
                } else {
                    ui.label("请先选择要播放的宏");
                }
            });

            // 宏间隔设置
            if !self.state.get_selected_macros().is_empty() {
                ui.separator();
                ui.label(format!("已选择 {} 个宏", self.state.get_selected_macros().len()));

                ui.horizontal(|ui| {
                    ui.label("宏间隔:");
                    let mut interval = self.state.get_macro_interval_ms();
                    if ui
                        .add(
                            egui::DragValue::new(&mut interval)
                                .speed(100)
                                .range(0..=600000)
                                .suffix("ms"),
                        )
                        .changed()
                    {
                        self.state.set_macro_interval_ms(interval);
                    }
                });
            }
        });
    }

    // 状态信息区域
    fn render_status_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let events_count = self.state.recorder.get_events_count();
                ui.label(format!("录制事件: {events_count}"));

                ui.separator();

                let total_macros = self.state.macro_manager.get_all_macros().len();
                ui.label(format!("已保存宏: {total_macros}"));

                ui.separator();

                // 显示快捷键提示
                ui.label("💡 按 F1 查看快捷键帮助");
            });
        });
    }

    fn render_help_panel(&mut self, ctx: &egui::Context) {
        egui::Window::new("快捷键帮助")
            .collapsible(true)
            .resizable(true)
            .default_size([400.0, 300.0])
            .show(ctx, |ui| {
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for shortcut in &self.shortcuts {
                        ui.horizontal(|ui| {
                            ui.label(&shortcut.description);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.colored_label(
                                        egui::Color32::LIGHT_BLUE,
                                        shortcut.display_text(),
                                    );
                                },
                            );
                        });
                    }
                });
            });
    }

    fn play_selected_macros(&mut self, repeat_count: u32) {
        self.state.play_selected_macros(
            &self.state.get_selected_macros(),
            repeat_count,
            self.state.get_macro_interval_ms(),
        );
    }
}
