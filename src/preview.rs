//! Canlı önizleme: tasarımı üretilen kodla aynı yardımcılarla çalıştırır.

use std::collections::HashMap;

use eframe::egui;

use crate::live;
use crate::model::{Document, Kind};

/// Önizlemede kullanıcının değiştirdiği değerler. Pencere her açılışta sıfırlanır.
#[derive(Default)]
pub struct LiveState {
    text: HashMap<u64, String>,
    flag: HashMap<u64, bool>,
    number: HashMap<u64, f32>,
    index: HashMap<u64, usize>,
    choice: HashMap<u64, Option<usize>>,
    groups: HashMap<String, usize>,
}

impl LiveState {
    /// Üretilen `App::ui` ile aynı işi yapar.
    pub fn render(&mut self, ui: &mut egui::Ui, doc: &Document) {
        ui.set_style(egui::Style {
            visuals: egui::Visuals::light(),
            ..egui::Style::default()
        });
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, ui.visuals().panel_fill);
        let o = ui.max_rect().min;
        let mut radio_index: HashMap<&str, usize> = HashMap::new();
        for w in &doc.widgets {
            let r = live::at(o, w.x, w.y, w.w, w.h);
            let id = format!("w{}", w.id);
            match w.kind {
                Kind::Label => {
                    live::label(ui, r, &w.text, w.align.egui());
                }
                Kind::Button => {
                    live::button(ui, r, &w.text);
                }
                Kind::Entry => {
                    let value = self.text.entry(w.id).or_insert_with(|| w.text.clone());
                    live::text_field(ui, r, value);
                }
                Kind::TextArea => {
                    let value = self.text.entry(w.id).or_insert_with(|| w.text.clone());
                    live::text_area(ui, r, &id, value);
                }
                Kind::Checkbox => {
                    let value = self.flag.entry(w.id).or_insert(w.checked);
                    live::checkbox(ui, r, value, &w.text);
                }
                Kind::Radio => {
                    let next = radio_index.entry(&w.group).or_default();
                    let index = *next;
                    *next += 1;
                    let value = self
                        .groups
                        .entry(w.group.clone())
                        .or_insert_with(|| doc.radio_initial(&w.group));
                    live::radio(ui, r, value, index, &w.text);
                }
                Kind::Combo => {
                    let items = w.clean_items();
                    let value = self.index.entry(w.id).or_insert(w.selected);
                    live::combo(ui, r, &id, value, &items);
                }
                Kind::List => {
                    let items = w.clean_items();
                    let value = self.choice.entry(w.id).or_insert(None);
                    live::list(ui, r, &id, value, &items);
                }
                Kind::Slider => {
                    let (lo, hi) = (w.min.min(w.max), w.max.max(w.min));
                    let value = self.number.entry(w.id).or_insert(w.value.clamp(lo, hi));
                    live::slider(ui, r, value, lo..=hi);
                }
                Kind::Progress => {
                    live::progress(ui, r, (w.value / 100.0).clamp(0.0, 1.0));
                }
                Kind::Group => live::group(ui, r, &w.text),
            }
        }
    }
}
