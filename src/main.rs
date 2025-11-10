#![windows_subsystem = "windows"]

use std::io::ErrorKind;

use eframe::egui::vec2;
use egui::ViewportBuilder;

use teste_timer::{Stage, Stages, TimerApp};

fn main() {
    let stages = match Stages::from_file("stages.json") {
        Ok(s) => s,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Stages::new(vec![
                Stage::new("Pausa", 5 * 60),
                Stage::new("Trabalha", 45 * 60),
            ]),
            _ => panic!("{e}"),
        },
    };

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder {
            inner_size: Some(vec2(300.0, 200.0)),
            resizable: Some(false),
            ..Default::default()
        },
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Timer Avançado",
        native_options,
        Box::new(|_cc| Ok(Box::new(TimerApp::new(stages)))),
    );
}
