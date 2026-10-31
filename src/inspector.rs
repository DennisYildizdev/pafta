//! Sağ panel: seçimin özellikleri.
//! Seçim yokken pencere, tek seçimde bileşen, çoklu seçimde hizalama.

use eframe::egui::{self, Align, Align2, Color32, Layout, Response, RichText, Sense, Ui, vec2};

use crate::app::Pafta;
use crate::i18n::Lang;
use crate::model::{AlignOp, HAlign, Kind, MIN_SIDE, MIN_WINDOW, ident, soft_ident, type_name};
use crate::panels::{Icon, caps, icon_button, segmented};
use crate::theme::{self, Tokens};

const LABEL_W: f32 = 72.0;
const SECTION_GAP: f32 = 28.0;

/// Panelde seçilen, çizimden sonra uygulanan işler.
enum Act {
    Align(AlignOp),
    Distribute(bool),
    MatchSize(bool),
    Raise,
    Lower,
    Duplicate,
    Delete,
    SelectRadio(u64),
}

fn header(ui: &mut Ui, t: &Tokens, dot: Option<Color32>, caption: &str, title: &str) {
    ui.horizontal(|ui| {
        if let Some(color) = dot {
            let (r, _) = ui.allocate_exact_size(vec2(8.0, 14.0), Sense::hover());
            ui.painter().circle_filled(r.center(), 3.5, color);
        }
        ui.label(caps(caption, t));
    });
    ui.add_space(2.0);
    let title = RichText::new(title)
        .font(theme::display(20.0))
        .color(t.text)
        .extra_letter_spacing(-0.3);
    ui.add(egui::Label::new(title).truncate());
}

/// Solda sabit genişlikte etiket, sağda denetim. Etiket, ekran okuyucu için denetimin adı olur.
fn row<R>(ui: &mut Ui, t: &Tokens, label: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.horizontal(|ui| labelled(ui, t, label, add)).inner
}

/// Çok satırlı denetimler için: etiket üstte hizalı.
fn row_top<R>(ui: &mut Ui, t: &Tokens, label: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.horizontal_top(|ui| labelled(ui, t, label, add)).inner
}

fn labelled<R>(ui: &mut Ui, t: &Tokens, label: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    let (r, _) = ui.allocate_exact_size(vec2(LABEL_W, ui.spacing().interact_size.y), Sense::hover());
    ui.painter().text(r.left_center(), Align2::LEFT_CENTER, label, theme::body(13.0), t.text2);
    if label.is_empty() {
        add(ui)
    } else {
        ui.name_inputs(label, add)
    }
}

fn text_field(ui: &mut Ui, value: &mut String) -> Response {
    ui.add(
        egui::TextEdit::singleline(value)
            .desired_width(f32::INFINITY)
            .align(Align2::LEFT_CENTER),
    )
}

/// Yan yana iki sayı alanı, örneğin X ve Y.
fn pair(ui: &mut Ui, t: &Tokens, label: &str, a: (&str, &mut f32), b: (&str, &mut f32), min: f32) {
    row(ui, t, label, |ui| {
        let width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
        let height = ui.spacing().interact_size.y;
        for (prefix, value) in [a, b] {
            let field = egui::DragValue::new(value)
                .speed(1.0)
                .range(min..=9999.0)
                .max_decimals(0)
                .prefix(RichText::new(prefix).color(t.text3))
                .min_size(vec2(width, height));
            let name = format!("{label} {}", prefix.trim());
            if ui.name_inputs(name, |ui| ui.add(field)).changed() {
                *value = value.round();
            }
        }
    });
}

fn section(ui: &mut Ui, t: &Tokens, title: &str) {
    ui.add_space(SECTION_GAP);
    ui.label(caps(title, t));
    ui.add_space(4.0);
}

fn align_buttons(ui: &mut Ui, t: &Tokens, l: Lang, acts: &mut Vec<Act>) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for (icon, op, tip) in [
            (Icon::AlignLeft, AlignOp::Left, l.pick("Align left", "Sola hizala")),
            (Icon::AlignCenterX, AlignOp::CenterX, l.pick("Center horizontally", "Yatayda ortala")),
            (Icon::AlignRight, AlignOp::Right, l.pick("Align right", "Sağa hizala")),
            (Icon::AlignTop, AlignOp::Top, l.pick("Align top", "Üste hizala")),
            (Icon::AlignCenterY, AlignOp::CenterY, l.pick("Center vertically", "Dikeyde ortala")),
            (Icon::AlignBottom, AlignOp::Bottom, l.pick("Align bottom", "Alta hizala")),
        ] {
            if icon_button(ui, t, icon, tip).clicked() {
                acts.push(Act::Align(op));
            }
        }
    });
}

fn order_and_actions(ui: &mut Ui, t: &Tokens, l: Lang, acts: &mut Vec<Act>) {
    row(ui, t, l.pick("Order", "Sıra"), |ui| {
        let forward = l.pick("PgUp · to front: Shift+PgUp", "PgUp · en öne: Shift+PgUp");
        if ui.button(l.pick("Bring forward", "Öne al")).on_hover_text(forward).clicked() {
            acts.push(Act::Raise);
        }
        let backward = l.pick("PgDn · to back: Shift+PgDn", "PgDn · en arkaya: Shift+PgDn");
        if ui.button(l.pick("Send backward", "Arkaya al")).on_hover_text(backward).clicked() {
            acts.push(Act::Lower);
        }
    });
    ui.add_space(SECTION_GAP);
    ui.horizontal(|ui| {
        if ui.button(l.pick("Duplicate", "Çoğalt")).on_hover_text("Ctrl+D").clicked() {
            acts.push(Act::Duplicate);
        }
        let delete = egui::Button::new(RichText::new(l.pick("Delete", "Sil")).color(t.danger));
        if ui.add(delete).on_hover_text("Delete").clicked() {
            acts.push(Act::Delete);
        }
    });
}

impl Pafta {
    pub(crate) fn inspector(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.ctx());
        let mut acts = Vec::new();
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            match self.sel.len() {
                0 => self.window_inspector(ui, &t),
                1 => self.widget_inspector(ui, &t, self.sel[0], &mut acts),
                _ => self.multi_inspector(ui, &t, &mut acts),
            }
        });
        for act in acts {
            self.apply(act);
        }
    }

    fn apply(&mut self, act: Act) {
        match act {
            Act::Align(op) => self.doc.align(&self.sel, op),
            Act::Distribute(horizontal) => self.doc.distribute(&self.sel, horizontal),
            Act::MatchSize(width) => self.doc.match_size(&self.sel, width),
            Act::Raise => self.doc.raise(&self.sel),
            Act::Lower => self.doc.lower(&self.sel),
            Act::Duplicate => self.duplicate(),
            Act::Delete => self.delete(),
            Act::SelectRadio(id) => self.doc.select_radio(id),
        }
    }

    fn window_inspector(&mut self, ui: &mut Ui, t: &Tokens) {
        let l = self.lang();
        header(ui, t, None, l.pick("WINDOW", "PENCERE"), &self.doc.title);
        section(ui, t, l.pick("CONTENT", "İÇERİK"));
        row(ui, t, l.pick("Title", "Başlık"), |ui| text_field(ui, &mut self.doc.title));
        row(ui, t, l.pick("Code name", "Kod adı"), |ui| {
            let name = RichText::new(type_name(&self.doc.title, l))
                .font(theme::mono(12.0))
                .color(t.text2);
            ui.add(egui::Label::new(name).truncate());
        });
        let doc = &mut self.doc;
        let (w, h) = (l.pick("W ", "En "), l.pick("H ", "Boy "));
        let min = MIN_WINDOW.x.min(MIN_WINDOW.y);
        pair(ui, t, l.pick("Size", "Boyut"), (w, &mut doc.width), (h, &mut doc.height), min);
        doc.width = doc.width.max(MIN_WINDOW.x);
        doc.height = doc.height.max(MIN_WINDOW.y);

        section(ui, t, l.pick("GRID", "IZGARA"));
        let show = l.pick("Show grid", "Izgarayı göster");
        row(ui, t, "", |ui| ui.checkbox(&mut self.prefs.show_grid, show));
        let snap = l.pick("Snap to grid and edges", "Izgaraya ve kenarlara yasla");
        row(ui, t, "", |ui| ui.checkbox(&mut self.prefs.snap, snap));
        row(ui, t, l.pick("Spacing", "Aralık"), |ui| {
            ui.add(
                egui::DragValue::new(&mut self.prefs.grid)
                    .range(2.0..=64.0)
                    .speed(0.25)
                    .max_decimals(0)
                    .suffix(" px"),
            )
        });

        ui.add_space(SECTION_GAP);
        let hint = l.pick(
            "Click a component to select it. Hold Shift to select more, or drag a box from an empty spot. Hold Alt to move freely and see distances.",
            "Bir bileşeni seçmek için tıkla, birden çoğu için Shift'e basılı tut ya da boş yerden çerçeve çiz. Alt basılıyken yaslama kapanır ve ölçüler görünür.",
        );
        ui.label(RichText::new(hint).color(t.text2));
    }

    fn widget_inspector(&mut self, ui: &mut Ui, t: &Tokens, id: u64, acts: &mut Vec<Act>) {
        let l = self.lang();
        let Some(index) = self.doc.index_of(id) else {
            return;
        };
        let kind = self.doc.widgets[index].kind;
        let name_clash = self.doc.name_taken(&self.doc.widgets[index].name, id);
        let w = &mut self.doc.widgets[index];
        header(ui, t, Some(t.family(kind.family())), kind.caption(l), &w.name);

        section(ui, t, l.pick("CONTENT", "İÇERİK"));
        row(ui, t, l.pick("Name", "Ad"), |ui| {
            let response = text_field(ui, &mut w.name);
            if response.changed() {
                w.name = soft_ident(&w.name);
            }
            if response.lost_focus() {
                w.name = ident(&w.name);
            }
        });
        if name_clash {
            let warning = l.pick(
                "Another component already uses this name.",
                "Bu ad başka bir bileşende de var.",
            );
            row(ui, t, "", |ui| {
                ui.label(RichText::new(warning).font(theme::body(11.0)).color(t.danger))
            });
        }
        let text_label = l.pick("Text", "Metin");
        let value_label = l.pick("Value", "Değer");
        match kind {
            Kind::Label => {
                row(ui, t, text_label, |ui| text_field(ui, &mut w.text));
                let options = [
                    (HAlign::Left, l.pick("Left", "Sol")),
                    (HAlign::Center, l.pick("Center", "Orta")),
                    (HAlign::Right, l.pick("Right", "Sağ")),
                ];
                row(ui, t, l.pick("Alignment", "Hizalama"), |ui| segmented(ui, t, &mut w.align, &options));
            }
            Kind::Button => {
                row(ui, t, text_label, |ui| text_field(ui, &mut w.text));
            }
            Kind::Group => {
                row(ui, t, l.pick("Title", "Başlık"), |ui| text_field(ui, &mut w.text));
            }
            Kind::Entry => {
                row(ui, t, value_label, |ui| text_field(ui, &mut w.text));
            }
            Kind::TextArea => {
                row_top(ui, t, value_label, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut w.text).desired_rows(4).desired_width(f32::INFINITY))
                });
            }
            Kind::Checkbox => {
                row(ui, t, text_label, |ui| text_field(ui, &mut w.text));
                row(ui, t, "", |ui| ui.checkbox(&mut w.checked, l.pick("Checked", "İşaretli")));
            }
            Kind::Radio => {
                row(ui, t, text_label, |ui| text_field(ui, &mut w.text));
                row(ui, t, l.pick("Group", "Grup"), |ui| text_field(ui, &mut w.group));
                let checked = w.checked;
                let selected = l.pick("Selected in its group", "Grubunda seçili");
                if row(ui, t, "", |ui| ui.radio(checked, selected)).clicked() {
                    acts.push(Act::SelectRadio(id));
                }
            }
            Kind::Combo | Kind::List => {
                row_top(ui, t, l.pick("Items", "Öğeler"), |ui| {
                    let mut text = w.items.join("\n");
                    let edit = egui::TextEdit::multiline(&mut text)
                        .desired_rows(4)
                        .desired_width(f32::INFINITY)
                        .hint_text(l.pick("One item per line", "Her satıra bir öğe"));
                    if ui.add(edit).changed() {
                        w.items = text.split('\n').map(str::to_owned).collect();
                    }
                });
                if kind == Kind::Combo {
                    let items: Vec<String> = w.clean_items().into_iter().map(str::to_owned).collect();
                    row(ui, t, l.pick("Selected", "Seçili"), |ui| {
                        let shown = items.get(w.selected).cloned().unwrap_or_default();
                        egui::ComboBox::from_id_salt("secili-oge")
                            .width(ui.available_width())
                            .selected_text(shown)
                            .show_ui(ui, |ui| {
                                for (i, item) in items.iter().enumerate() {
                                    ui.selectable_value(&mut w.selected, i, item.as_str());
                                }
                            });
                    });
                }
            }
            Kind::Slider => {
                let (lo_label, hi_label) = (l.pick("Min ", "En az "), l.pick("Max ", "En çok "));
                pair(ui, t, l.pick("Range", "Aralık"), (lo_label, &mut w.min), (hi_label, &mut w.max), -9999.0);
                let (lo, hi) = (w.min.min(w.max), w.max.max(w.min));
                row(ui, t, value_label, |ui| {
                    ui.add(egui::DragValue::new(&mut w.value).range(lo..=hi).speed(0.5).max_decimals(1))
                });
            }
            Kind::Progress => {
                let (prefix, suffix) = l.pick(("", "%"), ("%", ""));
                row(ui, t, value_label, |ui| {
                    ui.add(
                        egui::DragValue::new(&mut w.value)
                            .range(0.0..=100.0)
                            .speed(0.5)
                            .max_decimals(0)
                            .prefix(prefix)
                            .suffix(suffix),
                    )
                });
            }
        }

        section(ui, t, l.pick("LAYOUT", "YERLEŞİM"));
        pair(ui, t, l.pick("Position", "Konum"), ("X ", &mut w.x), ("Y ", &mut w.y), -9999.0);
        let (wl, hl) = (l.pick("W ", "En "), l.pick("H ", "Boy "));
        pair(ui, t, l.pick("Size", "Boyut"), (wl, &mut w.w), (hl, &mut w.h), MIN_SIDE);
        row(ui, t, l.pick("Align", "Hizala"), |ui| align_buttons(ui, t, l, acts));
        order_and_actions(ui, t, l, acts);
    }

    fn multi_inspector(&mut self, ui: &mut Ui, t: &Tokens, acts: &mut Vec<Act>) {
        let l = self.lang();
        let n = self.sel.len();
        let title = l.pick(format!("{n} components"), format!("{n} bileşen"));
        header(ui, t, None, l.pick("SELECTION", "SEÇİM"), &title);
        section(ui, t, l.pick("ALIGN", "HİZALA"));
        align_buttons(ui, t, l, acts);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.add_enabled_ui(n >= 3, |ui| {
                let tip = l.pick("Distribute horizontally", "Yatay boşlukları eşitle");
                if icon_button(ui, t, Icon::DistributeX, tip).clicked() {
                    acts.push(Act::Distribute(true));
                }
                let tip = l.pick("Distribute vertically", "Dikey boşlukları eşitle");
                if icon_button(ui, t, Icon::DistributeY, tip).clicked() {
                    acts.push(Act::Distribute(false));
                }
            });
            ui.add_space(8.0);
            let tip = l.pick("Match widths to the first selected", "Enleri ilk seçilene eşitle");
            if ui.button(l.pick("Width", "En")).on_hover_text(tip).clicked() {
                acts.push(Act::MatchSize(true));
            }
            let tip = l.pick("Match heights to the first selected", "Boyları ilk seçilene eşitle");
            if ui.button(l.pick("Height", "Boy")).on_hover_text(tip).clicked() {
                acts.push(Act::MatchSize(false));
            }
        });
        ui.add_space(SECTION_GAP);
        order_and_actions(ui, t, l, acts);
        ui.add_space(SECTION_GAP);
        let hint = l.pick(
            "With one component selected, alignment is relative to the window; with several, to each other.",
            "Tek bileşen seçince pencereye göre, birden çoğunu seçince birbirlerine göre hizalanır.",
        );
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.label(RichText::new(hint).color(t.text2));
        });
    }
}
