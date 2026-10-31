//! Üst çubuk, bileşen paleti, katmanlar, kod paneli ve küçük arayüz parçaları.

use std::sync::Arc;

use eframe::egui::{
    self, Align, Align2, Color32, Galley, Layout, Painter, Rect, Response, RichText, Sense, Stroke,
    StrokeKind, ThemePreference, Ui, UiBuilder, Vec2, pos2, vec2,
    text::{LayoutJob, TextFormat, TextWrapping},
};

use crate::app::{Pafta, Pending};
use crate::codegen;
use crate::highlight::{self, Palette};
use crate::i18n::Lang;
use crate::model::{Kind, Widget};
use crate::theme::{self, Tokens};

/// Küçük, büyük harfli bölüm başlığı: biraz açık harf aralığıyla.
pub(crate) fn caps(text: &str, t: &Tokens) -> RichText {
    RichText::new(text)
        .font(theme::semibold(11.0))
        .color(t.text2)
        .extra_letter_spacing(0.6)
}

pub(crate) fn caps_galley(painter: &Painter, text: &str, color: Color32) -> Arc<Galley> {
    let mut job = LayoutJob::default();
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: theme::semibold(11.0),
            color,
            extra_letter_spacing: 0.6,
            ..Default::default()
        },
    );
    painter.layout_job(job)
}

/// Ana eylem düğmesi: tek mavi vurgu.
pub(crate) fn primary_button(ui: &mut Ui, t: &Tokens, text: &str) -> Response {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), theme::semibold(13.0), t.on_accent);
    let size = vec2(galley.size().x + 24.0, ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, text));
    let fill = if response.hovered() && !response.is_pointer_button_down_on() {
        t.accent_hover
    } else {
        t.accent
    };
    ui.painter().rect_filled(rect, 8.0, fill);
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, t.on_accent);
    response
}

/// Menü satırı: tıklanınca menüyü kapatır.
fn item(ui: &mut Ui, text: &str, shortcut: &str) -> bool {
    let clicked = ui
        .add(egui::Button::new(text).shortcut_text(shortcut))
        .clicked();
    if clicked {
        ui.close();
    }
    clicked
}

#[derive(Clone, Copy)]
pub(crate) enum Icon {
    AlignLeft,
    AlignCenterX,
    AlignRight,
    AlignTop,
    AlignCenterY,
    AlignBottom,
    DistributeX,
    DistributeY,
}

/// İnce çizgili, kare simge düğmesi.
pub(crate) fn icon_button(ui: &mut Ui, t: &Tokens, icon: Icon, tip: &str) -> Response {
    let side = ui.spacing().interact_size.y + 2.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), tip));
    if response.hovered() {
        ui.painter().rect_filled(rect, 6.0, t.hover);
    }
    let color = if response.hovered() { t.text } else { t.text2 };
    paint_icon(ui.painter(), Rect::from_center_size(rect.center(), Vec2::splat(16.0)), icon, color);
    response.on_hover_text(tip)
}

fn paint_icon(p: &Painter, r: Rect, icon: Icon, color: Color32) {
    let line = Stroke::new(1.5, color);
    let bar = |x0: f32, y0: f32, x1: f32, y1: f32| {
        let b = Rect::from_min_max(r.min + vec2(x0, y0), r.min + vec2(x1, y1));
        p.rect_filled(b, 1.0, color);
    };
    let v = |x: f32| p.vline(r.left() + x, r.top() + 1.0..=r.bottom() - 1.0, line);
    let h = |y: f32| p.hline(r.left() + 1.0..=r.right() - 1.0, r.top() + y, line);
    match icon {
        Icon::AlignLeft => {
            bar(4.0, 3.5, 14.0, 7.0);
            bar(4.0, 9.0, 10.0, 12.5);
            v(1.5);
        }
        Icon::AlignCenterX => {
            bar(2.5, 3.5, 13.5, 7.0);
            bar(4.5, 9.0, 11.5, 12.5);
            v(8.0);
        }
        Icon::AlignRight => {
            bar(2.0, 3.5, 12.0, 7.0);
            bar(6.0, 9.0, 12.0, 12.5);
            v(14.5);
        }
        Icon::AlignTop => {
            bar(3.5, 4.0, 7.0, 14.0);
            bar(9.0, 4.0, 12.5, 10.0);
            h(1.5);
        }
        Icon::AlignCenterY => {
            bar(3.5, 2.5, 7.0, 13.5);
            bar(9.0, 4.5, 12.5, 11.5);
            h(8.0);
        }
        Icon::AlignBottom => {
            bar(3.5, 2.0, 7.0, 12.0);
            bar(9.0, 6.0, 12.5, 12.0);
            h(14.5);
        }
        Icon::DistributeX => {
            bar(6.0, 4.0, 10.0, 12.0);
            v(1.5);
            v(14.5);
        }
        Icon::DistributeY => {
            bar(4.0, 6.0, 12.0, 10.0);
            h(1.5);
            h(14.5);
        }
    }
}

/// macOS'taki gibi bölümlü seçici.
pub(crate) fn segmented<T: PartialEq + Copy>(
    ui: &mut Ui,
    t: &Tokens,
    value: &mut T,
    options: &[(T, &str)],
) -> bool {
    let height = ui.spacing().interact_size.y;
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    ui.painter().rect_filled(rect, 8.0, t.control);
    let width = rect.width() / options.len() as f32;
    let mut changed = false;
    for (i, (option, label)) in options.iter().enumerate() {
        let cell = Rect::from_min_size(rect.min + vec2(width * i as f32, 0.0), vec2(width, height)).shrink(2.0);
        let hit = ui.interact(cell, response.id.with(i), Sense::click());
        let on = *value == *option;
        hit.widget_info(|| egui::WidgetInfo::selected(egui::Role::RadioButton, true, on, *label));
        if on {
            ui.painter().rect(cell, 6.0, t.raised, Stroke::new(1.0, t.border), StrokeKind::Inside);
        } else if hit.hovered() {
            ui.painter().rect_filled(cell, 6.0, t.control_hover);
        }
        let color = if on { t.text } else { t.text2 };
        ui.painter().text(cell.center(), Align2::CENTER_CENTER, *label, theme::body(13.0), color);
        if hit.clicked() && !on {
            *value = *option;
            changed = true;
        }
    }
    changed
}

/// Etkin ya da devre dışı olabilen menü satırı; tıklanınca menüyü kapatır.
fn item_if(ui: &mut Ui, enabled: bool, text: &str, shortcut: &str) -> bool {
    let clicked = ui
        .add_enabled(enabled, egui::Button::new(text).shortcut_text(shortcut))
        .clicked();
    if clicked {
        ui.close();
    }
    clicked
}

impl Pafta {
    pub(crate) fn top_bar(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.ctx());
        let l = self.lang();
        let rect = ui.max_rect();

        // Ortada belge adı; kaydedilmemişse yanında küçük bir nokta.
        let galley = ui
            .painter()
            .layout_no_wrap(self.file_label(), theme::semibold(13.0), t.text);
        let pos = rect.center() - galley.size() / 2.0;
        if self.is_dirty() {
            let dot = pos2(pos.x + galley.size().x + 9.0, rect.center().y);
            ui.painter().circle_filled(dot, 3.0, t.text3);
        }
        ui.painter().galley(pos, galley, t.text);

        ui.scope_builder(
            UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                // Menü başlıkları çerçevesiz; yalnız üstüne gelince zemin alır.
                let inactive = &mut ui.visuals_mut().widgets.inactive;
                inactive.weak_bg_fill = Color32::TRANSPARENT;
                inactive.bg_stroke = Stroke::NONE;
                self.file_menu(ui);
                self.edit_menu(ui);
                self.view_menu(ui);
            },
        );
        ui.scope_builder(
            UiBuilder::new().max_rect(rect).layout(Layout::right_to_left(Align::Center)),
            |ui| {
                if primary_button(ui, &t, l.pick("Export…", "Dışa aktar…"))
                    .on_hover_text(l.pick("Save the code as main.rs (Ctrl+E)", "Kodu main.rs olarak kaydet (Ctrl+E)"))
                    .clicked()
                {
                    self.export_file();
                }
                let preview_tip = l.pick(
                    "Run the design with real egui widgets (F5)",
                    "Tasarımı gerçek egui bileşenleriyle çalıştır (F5)",
                );
                if ui.button(l.pick("Preview", "Önizle")).on_hover_text(preview_tip).clicked() {
                    self.open_preview();
                }
                let open = self.prefs.code_open;
                if ui
                    .add(egui::Button::selectable(open, l.pick("Code", "Kod")))
                    .on_hover_text(l.pick("Code panel (Ctrl+J)", "Kod paneli (Ctrl+J)"))
                    .clicked()
                {
                    self.prefs.code_open = !open;
                }
            },
        );
    }

    fn file_menu(&mut self, ui: &mut Ui) {
        let l = self.lang();
        ui.menu_button(l.pick("File", "Dosya"), |ui| {
            if item(ui, l.pick("New", "Yeni"), "Ctrl+N") {
                self.request(Pending::New);
            }
            if item(ui, l.pick("Open…", "Aç…"), "Ctrl+O") {
                self.request(Pending::Open);
            }
            ui.separator();
            if item(ui, l.pick("Save", "Kaydet"), "Ctrl+S") {
                self.save();
            }
            if item(ui, l.pick("Save As…", "Farklı kaydet…"), "Ctrl+Shift+S") {
                self.save_as();
            }
            ui.separator();
            if item(ui, l.pick("Export main.rs…", "main.rs olarak dışa aktar…"), "Ctrl+E") {
                self.export_file();
            }
            if item(ui, l.pick("Export Cargo Project…", "Cargo projesi olarak dışa aktar…"), "") {
                self.export_project();
            }
            ui.separator();
            if item(ui, l.pick("Open Example Design", "Örnek tasarımı aç"), "") {
                self.request(Pending::Example);
            }
        });
    }

    fn edit_menu(&mut self, ui: &mut Ui) {
        let l = self.lang();
        ui.menu_button(l.pick("Edit", "Düzen"), |ui| {
            let has_sel = !self.sel.is_empty();
            if item_if(ui, self.history.can_undo(&self.doc), l.pick("Undo", "Geri al"), "Ctrl+Z") {
                self.undo();
            }
            if item_if(ui, self.history.can_redo(), l.pick("Redo", "Yinele"), "Ctrl+Y") {
                self.redo();
            }
            ui.separator();
            if item_if(ui, has_sel, l.pick("Copy", "Kopyala"), "Ctrl+C") {
                self.copy(ui.ctx());
            }
            if item_if(ui, self.has_clip(), l.pick("Paste", "Yapıştır"), "Ctrl+V") {
                self.paste_clip();
            }
            if item_if(ui, has_sel, l.pick("Duplicate", "Çoğalt"), "Ctrl+D") {
                self.duplicate();
            }
            if item_if(ui, has_sel, l.pick("Delete", "Sil"), "Delete") {
                self.delete();
            }
            ui.separator();
            if item(ui, l.pick("Select All", "Tümünü seç"), "Ctrl+A") {
                self.select_all();
            }
            ui.separator();
            if item_if(ui, has_sel, l.pick("Bring Forward", "Öne al"), "PgUp") {
                self.doc.raise(&self.sel);
            }
            if item_if(ui, has_sel, l.pick("Send Backward", "Arkaya al"), "PgDn") {
                self.doc.lower(&self.sel);
            }
        });
    }

    fn view_menu(&mut self, ui: &mut Ui) {
        let l = self.lang();
        ui.menu_button(l.pick("View", "Görünüm"), |ui| {
            let t = theme::tokens(ui.ctx());
            if item(ui, l.pick("Zoom In", "Yakınlaştır"), "Ctrl++") {
                self.zoom_step(1, None);
            }
            if item(ui, l.pick("Zoom Out", "Uzaklaştır"), "Ctrl+-") {
                self.zoom_step(-1, None);
            }
            if item(ui, l.pick("Actual Size", "Gerçek boyut"), "Ctrl+0") {
                self.zoom_reset();
            }
            if item(ui, l.pick("Fit Window", "Pencereyi sığdır"), "") {
                self.view.fit = true;
            }
            ui.separator();
            ui.checkbox(&mut self.prefs.show_grid, l.pick("Show Grid", "Izgarayı göster"));
            ui.checkbox(&mut self.prefs.snap, l.pick("Snap to Grid and Edges", "Izgaraya ve kenarlara yasla"));
            ui.separator();
            if item(ui, l.pick("Code Panel", "Kod paneli"), "Ctrl+J") {
                self.prefs.code_open = !self.prefs.code_open;
            }
            if item(ui, l.pick("Preview", "Önizleme"), "F5") {
                self.open_preview();
            }
            if item(ui, l.pick("Shortcuts", "Kısayollar"), "F1") {
                self.help_open = true;
            }
            ui.separator();
            ui.label(caps(l.pick("THEME", "TEMA"), &t));
            let mut pref = ui.ctx().options(|o| o.theme_preference);
            let before = pref;
            ui.radio_value(&mut pref, ThemePreference::System, l.pick("System", "Sistem"));
            ui.radio_value(&mut pref, ThemePreference::Light, l.pick("Light", "Açık"));
            ui.radio_value(&mut pref, ThemePreference::Dark, l.pick("Dark", "Koyu"));
            if pref != before {
                ui.ctx().set_theme(pref);
            }
            ui.separator();
            // Diller kendi adlarıyla yazılır; yanlış dildeki kullanıcı da bulabilsin.
            ui.label(caps(l.pick("LANGUAGE", "DİL"), &t));
            ui.radio_value(&mut self.prefs.lang, Lang::En, "English");
            ui.radio_value(&mut self.prefs.lang, Lang::Tr, "Türkçe");
        });
    }

    pub(crate) fn left_panel(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.ctx());
        let l = self.lang();
        ui.label(caps(l.pick("COMPONENTS", "BİLEŞENLER"), &t));
        ui.add_space(6.0);
        ui.spacing_mut().item_spacing.y = 0.0;
        let mut family = None;
        for kind in Kind::ALL {
            if family.is_some_and(|f| f != kind.family()) {
                ui.add_space(6.0);
            }
            family = Some(kind.family());
            self.palette_row(ui, &t, kind);
        }

        ui.add_space(28.0);
        ui.horizontal(|ui| {
            ui.label(caps(l.pick("LAYERS", "KATMANLAR"), &t));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let count = self.doc.widgets.len().to_string();
                ui.label(RichText::new(count).font(theme::body(11.0)).color(t.text3));
            });
        });
        ui.add_space(6.0);
        if self.doc.widgets.is_empty() {
            let empty = l.pick("No components yet.", "Henüz bileşen yok.");
            ui.label(RichText::new(empty).color(t.text2));
            return;
        }
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| self.layers(ui, &t));
    }

    fn palette_row(&mut self, ui: &mut Ui, t: &Tokens, kind: Kind) {
        let l = self.lang();
        let (rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::click_and_drag());
        let armed = self.tool == Some(kind);
        let title = kind.title(l);
        let key = kind.key(l).symbol_or_name();
        response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, armed, title));
        let painter = ui.painter();
        if armed {
            painter.rect_filled(rect, 6.0, t.accent_tint);
        } else if response.hovered() {
            painter.rect_filled(rect, 6.0, t.hover);
        }
        painter.circle_filled(rect.left_center() + vec2(12.0, 0.0), 3.5, t.family(kind.family()));
        let color = if armed { t.accent } else { t.text };
        painter.text(rect.left_center() + vec2(26.0, 0.0), Align2::LEFT_CENTER, title, theme::body(13.0), color);
        let cap = Rect::from_center_size(rect.right_center() - vec2(16.0, 0.0), Vec2::splat(18.0));
        painter.rect_stroke(cap, 4.0, Stroke::new(1.0, t.border), StrokeKind::Inside);
        painter.text(cap.center(), Align2::CENTER_CENTER, key, theme::body(11.0), t.text2);

        if response.drag_started() {
            self.palette_drag = Some(kind);
            self.tool = None;
        } else if response.double_clicked() {
            let size = kind.default_size();
            let center = self.doc.client_rect().center();
            let min = pos2((center.x - size.x / 2.0).round(), (center.y - size.y / 2.0).round());
            self.add_widget(kind, Rect::from_min_size(min, size));
        } else if response.clicked() {
            self.tool = if armed { None } else { Some(kind) };
        }
        response.on_hover_text(l.pick(
            format!("{title}: drag onto the canvas, or select it and draw ({key})"),
            format!("{title}: tuvale sürükle ya da seçip tuvalde çiz ({key})"),
        ));
    }

    fn layers(&mut self, ui: &mut Ui, t: &Tokens) {
        let l = self.lang();
        ui.spacing_mut().item_spacing.y = 0.0;
        let mut clicked = None;
        let mut edit = None;
        let mut hovered = None;
        for w in self.doc.widgets.iter().rev() {
            let canvas_hover = self.view.hover == Some(w.id);
            let response = layer_row(ui, t, l, w, self.sel.contains(&w.id), canvas_hover);
            if response.hovered() {
                hovered = Some(w.id);
            }
            if response.double_clicked() {
                edit = Some(w.id);
            } else if response.clicked() {
                clicked = Some(w.id);
            }
        }
        self.view.layer_hover = hovered;
        if let Some(id) = clicked {
            let additive = ui.input(|i| i.modifiers.shift || i.modifiers.command);
            if !additive {
                self.sel = vec![id];
            } else if let Some(pos) = self.sel.iter().position(|s| *s == id) {
                self.sel.remove(pos);
            } else {
                self.sel.push(id);
            }
        }
        if let Some(id) = edit {
            self.begin_edit(id);
        }
    }

    pub(crate) fn code_panel(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.ctx());
        let l = self.lang();
        self.refresh_code(&t);
        ui.horizontal(|ui| {
            ui.label(RichText::new("main.rs").font(theme::semibold(13.0)).color(t.text));
            let lines = self.code.text.lines().count();
            let info = format!("eframe {}  ·  {lines} {}", codegen::EFRAME, l.pick("lines", "satır"));
            ui.label(RichText::new(info).font(theme::body(11.0)).color(t.text3));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button(l.pick("Copy", "Kopyala")).clicked() {
                    ui.ctx().copy_text(self.code.text.clone());
                    self.notify(l.pick("Code copied to the clipboard.", "Kod panoya kopyalandı."));
                }
            });
        });
        ui.add_space(4.0);
        let job = self.code.job.clone();
        let mut layouter = move |ui: &Ui, _: &dyn egui::TextBuffer, _: f32| {
            ui.fonts_mut(|f| f.layout_job(job.clone()))
        };
        egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
            let mut code = self.code.text.as_str();
            ui.add(
                egui::TextEdit::multiline(&mut code)
                    .code_editor()
                    .frame(egui::Frame::NONE)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter),
            );
        });
    }

    /// Belge, dil ya da tema değiştiyse kodu ve renklendirmesini yeniler.
    fn refresh_code(&mut self, t: &Tokens) {
        let lang = self.lang();
        let stale = self
            .code
            .source
            .as_ref()
            .is_none_or(|(doc, l)| *l != lang || *doc != self.doc);
        if stale {
            self.code.text = codegen::generate(&self.doc, lang);
            self.code.source = Some((self.doc.clone(), lang));
            self.code.dark = None;
        }
        if self.code.dark != Some(t.dark) {
            let palette = Palette::new(t.dark, t.text);
            self.code.job = highlight::highlight(&self.code.text, theme::mono(12.0), &palette);
            self.code.dark = Some(t.dark);
        }
    }
}

fn layer_row(ui: &mut Ui, t: &Tokens, l: Lang, w: &Widget, selected: bool, canvas_hover: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::ListItem, true, selected, &w.name));
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, 6.0, t.accent_tint);
    } else if response.hovered() || canvas_hover {
        painter.rect_filled(rect, 6.0, t.hover);
    }
    painter.circle_filled(rect.left_center() + vec2(12.0, 0.0), 3.0, t.family(w.kind.family()));
    let kind = painter.layout_no_wrap(w.kind.title(l).to_owned(), theme::body(11.0), t.text3);
    let kind_pos = pos2(rect.right() - 8.0 - kind.size().x, rect.center().y - kind.size().y / 2.0);
    // Uzun adlar türün önünde üç noktayla kısalır.
    let name_left = rect.left() + 24.0;
    let mut job = LayoutJob::simple_singleline(w.name.clone(), theme::body(13.0), t.text);
    job.wrap = TextWrapping::truncate_at_width((kind_pos.x - 8.0 - name_left).max(0.0));
    let name = painter.layout_job(job);
    painter.galley(pos2(name_left, rect.center().y - name.size().y / 2.0), name, t.text);
    painter.galley(kind_pos, kind, t.text3);
    response
}
