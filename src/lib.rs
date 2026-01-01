use eframe::egui;
use egui::{Align, Layout, WidgetText};
use std::time::Duration;

mod stages;
mod timer;

pub use crate::stages::{Stage, Stages};
pub use crate::timer::Timer;

const FAST_REFRESH_MS: u64 = 64;
const SLOW_REFRESH_MS: u64 = 256;

pub struct TimerApp {
    timer: Timer,
    looping: bool,
    allowed_to_close: bool,
    showing_close_confirm: bool,
    showing_config: bool,
    editing_stage_index: Option<usize>,
    editing_text: String,
}
impl TimerApp {
    pub fn new(stages: Stages) -> TimerApp {
        TimerApp {
            timer: Timer::new(stages),
            showing_config: false,
            looping: false,
            allowed_to_close: false,
            showing_close_confirm: false,
            editing_stage_index: None,
            editing_text: String::new(),
        }
    }

    fn show_controls(&mut self, ui: &mut egui::Ui) {
        let layout = Layout::left_to_right(Align::Center);
        let play_text = if !self.timer.has_started() || self.timer.is_paused {
            "▶"
        } else {
            "⏸"
        };
        ui.vertical_centered(|ui| {
            let text = if self.looping {
                "🔄 Ciclar"
            } else {
                "◻ Ciclar"
            };
            ui.add_space(10.);
            add_button(ui, text, || self.looping = !self.looping);
        });
        ui.columns(9, |column| {
            column[3].with_layout(layout, |ui| {
                add_button(ui, play_text, || self.timer.toggle_play());
            });
            column[4].with_layout(layout, |ui| {
                add_button(ui, "⏹", || self.timer.reset(false));
            });
            column[5].with_layout(layout, |ui| {
                add_button(ui, "⟲", || self.timer.reset(true));
            });
        });
    }

    fn check_closing_request(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) && !self.allowed_to_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.showing_close_confirm = true
        }
        if !self.showing_close_confirm {
            return;
        }
        egui::Window::new("Confirm")
            .collapsible(false)
            .resizable(false)
            .fixed_size([200.0, 100.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                self.draw_close_confirmation_window(ctx, ui);
            });
    }

    fn draw_close_confirmation_window(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.label("Are you sure you want to quit?");
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            add_button(ui, "No", || self.showing_close_confirm = false);
            add_button(ui, "Yes", || {
                self.allowed_to_close = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            });
        });
    }

    fn draw_settings(&mut self, ui: &mut egui::Ui, time_left: u32) {
        ui.label(format_time(time_left));
        ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
            add_button(ui, "⚙", || self.showing_config = !self.showing_config);
        });
    }
}

// for configuration window
impl TimerApp {
    fn show_stages_config(&mut self, ctx: &egui::Context) {
        if !self.showing_config {
            return;
        }
        egui::Window::new("Estágios")
            .collapsible(false)
            .show(ctx, |ui| {
                self.draw_buttons(ui);
                self.draw_stages_list(ui);
            });
    }

    fn draw_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            add_button(ui, "➕", || self.timer.stages.add(Stage::default()));
            ui.add_space(57.);
            add_button(ui, "💾", || {
                self.timer
                    .stages
                    .to_file("stages.json")
                    .unwrap_or_else(|e| eprintln!("{e}"))
            });
        });
    }

    fn draw_stages_list(&mut self, ui: &mut egui::Ui) {
        let mut to_remove: Option<usize> = None;
        for stage_index in 0..self.timer.stages.len() {
            to_remove = self.draw_stage(ui, stage_index);
        }
        if let Some(index) = to_remove {
            self.timer.stages.remove(index);
        }
    }

    fn draw_stage(&mut self, ui: &mut egui::Ui, index: usize) -> Option<usize> {
        let stage = match self.timer.stages.get_mut(index) {
            Some(s) => s,
            None => return None,
        };
        let mut to_remove_index = false;

        ui.horizontal(|ui| {
            let mut mins = stage.duration.as_secs() / 60;
            add_button(ui, " × ", || to_remove_index = true);
            ui.add(egui::DragValue::new(&mut mins).range(1..=180))
                .changed()
                .then(|| stage.duration = Duration::from_secs(mins * 60));

            if Some(index) == self.editing_stage_index {
                ui.text_edit_singleline(&mut self.editing_text);
                ui.input(|i| {
                    i.key_pressed(egui::Key::Enter) //
                })
                .then(|| {
                    stage.name = self.editing_text.clone();
                    self.editing_stage_index = None;
                    self.editing_text.clear();
                });
                return;
            }
            ui.label(format!(": {}", stage.name))
                .double_clicked()
                .then(|| {
                    self.editing_stage_index = Some(index);
                    self.editing_text = stage.name.clone();
                });
        });

        match to_remove_index {
            true => Some(index),
            false => None,
        }
    }
}

impl eframe::App for TimerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.timer.check_progress(self.looping);

        self.check_closing_request(ctx);
        self.show_stages_config(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            let (stage_name, stage_duration) = match self.timer.stages.get_current() {
                Some(s) => (s.name, s.duration.as_secs_f32()),
                None => (String::from("Estágio"), 120.),
            };

            let time_left = self.timer.time_left().as_secs_f32();
            let progress = 1.0 - time_left / stage_duration;

            ui.add_space(20.);
            ui.vertical_centered(|ui| ui.heading(format!("{}", stage_name)));

            ui.horizontal(|ui| self.draw_settings(ui, time_left as u32));
            ui.add(egui::ProgressBar::new(progress));
            self.show_controls(ui);

            let delay = if !self.timer.is_paused && self.timer.has_started() {
                FAST_REFRESH_MS
            } else {
                SLOW_REFRESH_MS
            };
            ctx.request_repaint_after(Duration::from_millis(delay));
        });
    }
}

fn format_time(secs: u32) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn add_button<T: FnOnce()>(ui: &mut egui::Ui, text: impl Into<WidgetText>, f: T) {
    ui.button(text).clicked().then(f);
}
