use eframe::egui;
use egui::{Align, Layout, WidgetText};
use std::time::Duration;

mod stages;
mod timer;

pub use crate::stages::{Stage, Stages};
pub use crate::timer::Timer;

pub struct TimerApp {
    timer: Timer,
    show_config: bool,
    loop_is_enabled: bool,
    editing_stage_name: Option<usize>,
    editing_text: String,
}
impl TimerApp {
    pub fn new(stages: Stages) -> TimerApp {
        TimerApp {
            timer: Timer::new(stages),
            show_config: false,
            loop_is_enabled: false,
            editing_stage_name: None,
            editing_text: String::new(),
        }
    }

    fn show_controls(&mut self, ui: &mut egui::Ui) {
        let layout = Layout::left_to_right(Align::Center);
        let play_text = if !self.timer.has_stared() || self.timer.is_paused {
            "▶"
        } else {
            "⏸"
        };
        ui.vertical_centered(|ui| {
            let text = if self.loop_is_enabled {
                "🔄 Ciclar"
            } else {
                "◻ Ciclar"
            };
            ui.add_space(10.);
            add_button(ui, text, || self.loop_is_enabled = !self.loop_is_enabled);
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

    fn show_stages_config(&mut self, ctx: &egui::Context) {
        egui::Window::new("Estágios") // center s
            .collapsible(false)
            .show(ctx, |ui| {
                let mut to_remove: Option<usize> = None;

                for (i, stage) in self.timer.stages.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let mut mins = stage.duration.as_secs() / 60;
                        add_button(ui, " × ", || to_remove = Some(i));

                        ui.add(egui::DragValue::new(&mut mins).range(1..=180))
                            .changed()
                            .then(|| stage.duration = Duration::from_secs(mins * 60));

                        if Some(i) == self.editing_stage_name {
                            ui.text_edit_singleline(&mut self.editing_text);
                            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                stage.name = self.editing_text.clone();
                                self.editing_stage_name = None;
                                self.editing_text.clear();
                            }
                        } else {
                            ui.label(format!(": {}", stage.name))
                                .double_clicked()
                                .then(|| {
                                    self.editing_stage_name = Some(i);
                                    self.editing_text = stage.name.clone();
                                });
                        }
                    });
                }
                if let Some(index) = to_remove {
                    self.timer.stages.remove(index);
                }

                ui.horizontal(|ui| {
                    add_button(ui, "➕", || self.timer.stages.add(Stage::default()));
                    ui.add_space(57.);
                    add_button(ui, "💾", || {
                        match self.timer.stages.to_file("stages.json") {
                            Ok(_) => (),
                            Err(e) => eprintln!("{e}"),
                        }
                    });
                });
            });
    }
}

impl eframe::App for TimerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.timer.check_progress(self.loop_is_enabled);

        if self.show_config {
            self.show_stages_config(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let (stage_name, stage_duration) = match self.timer.stages.get_current() {
                Some(s) => (s.name, s.duration.as_secs_f32()),
                None => (String::from("Estágio"), 120.),
            };

            let time_left = self.timer.time_left().as_secs_f32();
            let progress = 1.0 - time_left / stage_duration;

            ui.add_space(20.);
            ui.vertical_centered(|ui| ui.heading(format!("{}", stage_name)));

            ui.horizontal(|ui| {
                ui.label(format_time(time_left as u32));
                ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                    add_button(ui, "⚙", || self.show_config = !self.show_config);
                });
            });
            ui.add(egui::ProgressBar::new(progress));
            self.show_controls(ui);

            if !self.timer.is_paused && self.timer.has_stared() {
                ctx.request_repaint_after(Duration::from_millis(64));
            } else {
                ctx.request_repaint_after(Duration::from_millis(256));
            }
        });
    }
}

fn format_time(secs: u32) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn add_button<T: FnOnce()>(ui: &mut egui::Ui, text: impl Into<WidgetText>, f: T) {
    ui.button(text).clicked().then(f);
}
