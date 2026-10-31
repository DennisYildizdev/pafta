//! Pafta: egui arayüzlerini elle tasarlama aracı.
//!
//! Bileşenleri bir pencereye sürükleyip yerleştirirsin, Pafta da tasarımı
//! `cargo run` ile çalışan bir eframe uygulamasına çevirir.

mod app;
mod canvas;
pub mod codegen;
mod highlight;
mod history;
pub mod i18n;
mod icon;
mod inspector;
pub mod live;
pub mod model;
mod panels;
mod preview;
mod snap;
mod theme;
#[cfg(test)]
mod ui_tests;

use eframe::egui;

/// Tasarım penceresini açar.
pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Pafta")
            .with_app_id("pafta")
            .with_inner_size([1320.0, 840.0])
            .with_min_inner_size([960.0, 600.0])
            .with_icon(theme::icon()),
        ..Default::default()
    };
    eframe::run_native(
        "Pafta",
        options,
        Box::new(|cc| Ok(Box::new(app::Pafta::new(cc)))),
    )
}
