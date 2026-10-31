//! Uygulama durumu: belge, seçim, kısayollar, dosya işlemleri ve pencere düzeni.

use std::path::{Path, PathBuf};
use std::time::Duration;

use eframe::egui::{
    self, Align, Align2, Color32, Id, Key, LayerId, Layout, Margin, Modifiers, Order, RichText,
    text::LayoutJob, vec2,
};
use serde::{Deserialize, Serialize};

use crate::canvas::View;
use crate::codegen;
use crate::history::History;
use crate::i18n::Lang;
use crate::model::{Document, Kind, Widget};
use crate::panels::primary_button;
use crate::preview::LiveState;
use crate::theme;

/// Açılışlar arasında saklanan ayarlar.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub grid: f32,
    pub snap: bool,
    pub show_grid: bool,
    pub code_open: bool,
    /// Arayüz dili; varsayılan İngilizce.
    pub lang: Lang,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            grid: 8.0,
            snap: true,
            show_grid: true,
            code_open: true,
            lang: Lang::En,
        }
    }
}

/// Kaydedilmemiş değişiklik varken sorulacak iş.
#[derive(Clone, Debug)]
pub enum Pending {
    New,
    Open,
    OpenPath(PathBuf),
    Example,
}

pub struct Toast {
    text: String,
    error: bool,
    until: f64,
}

/// Üretilen kod ve renklendirilmiş hâli; belge ya da dil değişince yenilenir.
#[derive(Default)]
pub struct CodeCache {
    pub source: Option<(Document, Lang)>,
    pub text: String,
    pub job: LayoutJob,
    pub dark: Option<bool>,
}

/// Kapanışta saklanır, açılışta kalınan yerden devam edilir.
#[derive(Serialize, Deserialize)]
struct Session {
    doc: Document,
    saved: Document,
    path: Option<PathBuf>,
    prefs: Prefs,
}

/// Panoya JSON olarak giden bileşenler; başka bir Pafta penceresine de yapıştırılabilir.
#[derive(Serialize, Deserialize)]
struct Clip {
    pafta: u32,
    widgets: Vec<Widget>,
}

/// Kısayollar penceresinin satırları: (iş, tuşlar).
fn shortcut_rows(l: Lang) -> Vec<(&'static str, String)> {
    let tools = Kind::ALL
        .iter()
        .map(|k| k.key(l).symbol_or_name())
        .collect::<Vec<_>>()
        .join(" ");
    let rows = [
        (l.pick("Drop the tool, clear selection", "Aracı bırak, seçimi kaldır"), "Esc"),
        (l.pick("Edit text in place", "Metni düzenle"), l.pick("Enter or double-click", "Enter ya da çift tık")),
        (l.pick("Move: 1 pixel, one grid step", "Taşı: 1 piksel, ızgara"), l.pick("Arrows, Shift + arrows", "Oklar, Shift + oklar")),
        (l.pick("Move without snapping, see distances", "Yaslamadan taşı, ölçüleri gör"), "Alt"),
        (l.pick("Duplicate", "Çoğalt"), "Ctrl+D"),
        (l.pick("Copy, paste, cut", "Kopyala, yapıştır, kes"), "Ctrl+C, Ctrl+V, Ctrl+X"),
        (l.pick("Delete", "Sil"), "Delete"),
        (l.pick("Undo, redo", "Geri al, yinele"), "Ctrl+Z, Ctrl+Y"),
        (l.pick("Bring forward, send backward", "Öne al, arkaya al"), l.pick("PgUp, PgDn (Shift: all the way)", "PgUp, PgDn (Shift: en uca)")),
        (l.pick("Zoom in, zoom out", "Yakınlaştır, uzaklaştır"), l.pick("Ctrl + wheel, Ctrl++, Ctrl+-", "Ctrl + tekerlek, Ctrl++, Ctrl+-")),
        (l.pick("Actual size", "Gerçek boyut"), "Ctrl+0"),
        (l.pick("Pan", "Kaydır"), l.pick("Wheel, Space + drag, middle button", "Tekerlek, Boşluk + sürükle, orta tuş")),
        (l.pick("Grid", "Izgara"), "Ctrl+G"),
        (l.pick("Code panel", "Kod paneli"), "Ctrl+J"),
        (l.pick("Preview", "Önizleme"), "F5"),
        (l.pick("New, open, save", "Yeni, aç, kaydet"), "Ctrl+N, Ctrl+O, Ctrl+S"),
        (l.pick("Export main.rs", "main.rs olarak dışa aktar"), "Ctrl+E"),
    ];
    std::iter::once((l.pick("Component tool", "Bileşen aracı"), tools))
        .chain(rows.into_iter().map(|(what, keys)| (what, keys.to_owned())))
        .collect()
}

pub struct Pafta {
    pub(crate) doc: Document,
    /// Son kaydedilen ya da açılan hâl: değişiklik var mı diye buna bakılır.
    saved: Document,
    path: Option<PathBuf>,
    pub(crate) history: History,
    pub(crate) sel: Vec<u64>,
    pub(crate) tool: Option<Kind>,
    pub(crate) palette_drag: Option<Kind>,
    pub(crate) view: View,
    pub(crate) prefs: Prefs,
    pub(crate) code: CodeCache,
    pub(crate) help_open: bool,
    pending: Option<Pending>,
    toast: Option<Toast>,
    preview_open: bool,
    live: LiveState,
    clip: Vec<Widget>,
    window_title: String,
    now: f64,
}

impl Pafta {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);
        let session: Option<Session> =
            cc.storage.and_then(|s| eframe::get_value(s, eframe::APP_KEY));
        let (doc, saved, path, prefs) = match session {
            Some(s) => (s.doc, s.saved, s.path, s.prefs),
            None => {
                let prefs = Prefs::default();
                let doc = Document::example(prefs.lang);
                (doc.clone(), doc, None, prefs)
            }
        };
        Self {
            history: History::new(&doc),
            doc,
            saved,
            path,
            sel: Vec::new(),
            tool: None,
            palette_drag: None,
            view: View::default(),
            prefs,
            code: CodeCache::default(),
            help_open: false,
            pending: None,
            toast: None,
            preview_open: false,
            live: LiveState::default(),
            clip: Vec::new(),
            window_title: String::new(),
            now: 0.0,
        }
    }

    /// Arayüz dili.
    pub(crate) fn lang(&self) -> Lang {
        self.prefs.lang
    }

    pub(crate) fn file_label(&self) -> String {
        let untitled = self.lang().pick("Untitled", "Adsız");
        self.path
            .as_deref()
            .and_then(Path::file_name)
            .map_or_else(|| untitled.to_owned(), |name| name.to_string_lossy().into_owned())
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.doc != self.saved
    }

    pub(crate) fn notify(&mut self, text: impl Into<String>) {
        self.toast = Some(Toast {
            text: text.into(),
            error: false,
            until: self.now + 2.5,
        });
    }

    fn notify_error(&mut self, text: impl Into<String>) {
        self.toast = Some(Toast {
            text: text.into(),
            error: true,
            until: self.now + 5.0,
        });
    }

    /// Kaydedilmemiş değişiklik varsa önce sorar.
    pub(crate) fn request(&mut self, pending: Pending) {
        if self.is_dirty() {
            self.pending = Some(pending);
        } else {
            self.run(pending);
        }
    }

    fn design_filter(&self) -> &'static str {
        self.lang().pick("Pafta design", "Pafta tasarımı")
    }

    fn run(&mut self, pending: Pending) {
        let l = self.lang();
        match pending {
            Pending::New => self.replace(Document::new(l), None),
            Pending::Example => self.replace(Document::example(l), None),
            Pending::Open => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter(self.design_filter(), &["pafta"])
                    .pick_file()
                {
                    self.open_path(&path);
                }
            }
            Pending::OpenPath(path) => self.open_path(&path),
        }
    }

    fn open_path(&mut self, path: &Path) {
        let l = self.lang();
        match read_doc(path, l) {
            Ok(doc) => {
                self.replace(doc, Some(path.to_path_buf()));
                let name = self.file_label();
                self.notify(l.pick(format!("{name} opened."), format!("{name} açıldı.")));
            }
            Err(e) => self.notify_error(l.pick(
                format!("Couldn't open the file: {e}"),
                format!("Dosya açılamadı: {e}"),
            )),
        }
    }

    /// Belgeyi değiştirir; geçmiş, seçim ve görünüm sıfırlanır.
    fn replace(&mut self, doc: Document, path: Option<PathBuf>) {
        self.history.reset(&doc);
        self.saved = doc.clone();
        self.doc = doc;
        self.path = path;
        self.sel.clear();
        self.tool = None;
        // Yeni görünüm ilk karede pencereyi tuvale sığdırır.
        self.view = View::default();
    }

    pub(crate) fn save(&mut self) -> bool {
        match self.path.clone() {
            Some(path) => self.write_to(&path),
            None => self.save_as(),
        }
    }

    pub(crate) fn save_as(&mut self) -> bool {
        let name = format!("{}.pafta", crate::model::ident(&self.doc.title));
        let Some(path) = rfd::FileDialog::new()
            .add_filter(self.design_filter(), &["pafta"])
            .set_file_name(name)
            .save_file()
        else {
            return false;
        };
        self.write_to(&path)
    }

    fn write_to(&mut self, path: &Path) -> bool {
        let l = self.lang();
        let result = serde_json::to_string_pretty(&self.doc)
            .map_err(std::io::Error::other)
            .and_then(|json| std::fs::write(path, json));
        match result {
            Ok(()) => {
                self.path = Some(path.to_path_buf());
                self.saved = self.doc.clone();
                let name = self.file_label();
                self.notify(l.pick(format!("{name} saved."), format!("{name} kaydedildi.")));
                true
            }
            Err(e) => {
                self.notify_error(l.pick(format!("Couldn't save: {e}"), format!("Kaydedilemedi: {e}")));
                false
            }
        }
    }

    pub(crate) fn export_file(&mut self) {
        let l = self.lang();
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Rust", &["rs"])
            .set_file_name("main.rs")
            .save_file()
        else {
            return;
        };
        match std::fs::write(&path, codegen::generate(&self.doc, l)) {
            Ok(()) => {
                let at = path.display();
                self.notify(l.pick(format!("Code written to {at}"), format!("Kod yazıldı: {at}")));
            }
            Err(e) => self.notify_error(l.pick(
                format!("Couldn't write the code: {e}"),
                format!("Kod yazılamadı: {e}"),
            )),
        }
    }

    /// Boş bir klasöre `Cargo.toml` ve `src/main.rs` yazar; `cargo run` ile hemen çalışır.
    pub(crate) fn export_project(&mut self) {
        let l = self.lang();
        let Some(dir) = rfd::FileDialog::new()
            .set_title(l.pick("Choose an empty folder", "Boş bir klasör seç"))
            .pick_folder()
        else {
            return;
        };
        let empty = std::fs::read_dir(&dir).is_ok_and(|mut entries| entries.next().is_none());
        if !empty {
            self.notify_error(l.pick(
                "The folder must be empty. Nothing was overwritten.",
                "Klasör boş olmalı. Hiçbir dosyanın üzerine yazılmadı.",
            ));
            return;
        }
        let code = codegen::generate(&self.doc, l);
        let result = std::fs::write(dir.join("Cargo.toml"), codegen::cargo_toml(&self.doc))
            .and_then(|()| std::fs::create_dir_all(dir.join("src")))
            .and_then(|()| std::fs::write(dir.join("src").join("main.rs"), code));
        match result {
            Ok(()) => {
                let at = dir.display();
                self.notify(l.pick(
                    format!("Project ready in {at}. Run it with: cargo run"),
                    format!("Proje hazır: {at}. Çalıştırmak için: cargo run"),
                ));
            }
            Err(e) => self.notify_error(l.pick(
                format!("Couldn't write the project: {e}"),
                format!("Proje yazılamadı: {e}"),
            )),
        }
    }

    pub(crate) fn open_preview(&mut self) {
        self.live = LiveState::default();
        self.preview_open = true;
    }

    pub(crate) fn undo(&mut self) {
        self.view.edit = None;
        self.history.undo(&mut self.doc);
    }

    pub(crate) fn redo(&mut self) {
        self.view.edit = None;
        self.history.redo(&mut self.doc);
    }

    pub(crate) fn select_all(&mut self) {
        self.sel = self.doc.widgets.iter().map(|w| w.id).collect();
    }

    pub(crate) fn copy(&mut self, ctx: &egui::Context) {
        let widgets: Vec<Widget> = self
            .doc
            .widgets
            .iter()
            .filter(|w| self.sel.contains(&w.id))
            .cloned()
            .collect();
        if widgets.is_empty() {
            return;
        }
        if let Ok(json) = serde_json::to_string(&Clip {
            pafta: 1,
            widgets: widgets.clone(),
        }) {
            ctx.copy_text(json);
        }
        let n = widgets.len();
        let text = match (self.lang(), n) {
            (Lang::En, 1) => "1 component copied.".to_owned(),
            (Lang::En, n) => format!("{n} components copied."),
            (Lang::Tr, n) => format!("{n} bileşen kopyalandı."),
        };
        self.notify(text);
        self.clip = widgets;
    }

    pub(crate) fn has_clip(&self) -> bool {
        !self.clip.is_empty()
    }

    pub(crate) fn paste_clip(&mut self) {
        let widgets = self.clip.clone();
        self.paste(widgets);
    }

    fn paste(&mut self, widgets: Vec<Widget>) {
        if widgets.is_empty() {
            return;
        }
        self.sel = self.doc.insert_copies(widgets, vec2(16.0, 16.0));
        self.tool = None;
    }

    /// Panodan gelen metin: Pafta bileşenleri ya da düz metin (yeni bir etiket olur).
    fn paste_text(&mut self, text: &str) {
        if let Ok(clip) = serde_json::from_str::<Clip>(text)
            && clip.pafta == 1
        {
            self.paste(clip.widgets);
            return;
        }
        let line = text.lines().next().unwrap_or_default().trim();
        if line.is_empty() {
            return;
        }
        let size = Kind::Label.default_size();
        let center = self.doc.client_rect().center();
        let rect = egui::Rect::from_center_size(center, size).round();
        self.add_widget(Kind::Label, rect);
        if let Some(w) = self.sel.first().and_then(|id| self.doc.get_mut(*id)) {
            w.text = line.to_owned();
        }
    }

    pub(crate) fn duplicate(&mut self) {
        if self.sel.is_empty() {
            return;
        }
        let step = self.prefs.grid * 2.0;
        self.sel = self.doc.duplicate(&self.sel, vec2(step, step));
    }

    pub(crate) fn delete(&mut self) {
        let ids = std::mem::take(&mut self.sel);
        self.doc.remove(&ids);
        self.view.edit = None;
    }

    fn nudge(&mut self, dx: f32, dy: f32) {
        for id in self.move_set() {
            if let Some(w) = self.doc.get_mut(id) {
                let r = w.rect();
                w.set_rect(r.translate(vec2(dx, dy)));
            }
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        let typing = ctx.text_edit_focused();
        let cmd = |key| ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, key));
        let cmd_shift = |key| ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, key));
        let plain = |key| ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key));

        // Ctrl+Shift kısayolları önce: Ctrl+S, Ctrl+Shift+S'ye de uyar.
        if cmd_shift(Key::S) {
            self.save_as();
        }
        if cmd(Key::S) {
            self.save();
        }
        if cmd(Key::O) {
            self.request(Pending::Open);
        }
        if cmd(Key::N) {
            self.request(Pending::New);
        }
        if cmd(Key::E) {
            self.export_file();
        }
        if cmd(Key::J) {
            self.prefs.code_open = !self.prefs.code_open;
        }
        if plain(Key::F5) {
            self.open_preview();
        }
        if plain(Key::F1) {
            self.help_open = !self.help_open;
        }
        if typing || self.pending.is_some() || self.help_open {
            return;
        }

        if cmd_shift(Key::Z) || cmd(Key::Y) {
            self.redo();
        }
        if cmd(Key::Z) {
            self.undo();
        }
        if cmd(Key::A) {
            self.select_all();
        }
        if cmd(Key::D) {
            self.duplicate();
        }
        if cmd(Key::G) {
            let on = !(self.prefs.show_grid && self.prefs.snap);
            self.prefs.show_grid = on;
            self.prefs.snap = on;
        }
        if cmd(Key::Plus) || cmd(Key::Equals) {
            self.zoom_step(1, None);
        }
        if cmd(Key::Minus) {
            self.zoom_step(-1, None);
        }
        if cmd(Key::Num0) {
            self.zoom_reset();
        }

        let events = ctx.input(|i| i.events.clone());
        for event in &events {
            match event {
                egui::Event::Copy => self.copy(ctx),
                egui::Event::Cut => {
                    self.copy(ctx);
                    self.delete();
                }
                egui::Event::Paste(text) => self.paste_text(text),
                _ => {}
            }
        }

        let shift = ctx.input(|i| i.modifiers.shift);
        let step = if shift { self.prefs.grid } else { 1.0 };
        let pressed = |key| ctx.input(|i| i.key_pressed(key));
        if !self.sel.is_empty() {
            if pressed(Key::Delete) || pressed(Key::Backspace) {
                self.delete();
            }
            for (key, dx, dy) in [
                (Key::ArrowLeft, -step, 0.0),
                (Key::ArrowRight, step, 0.0),
                (Key::ArrowUp, 0.0, -step),
                (Key::ArrowDown, 0.0, step),
            ] {
                if pressed(key) {
                    self.nudge(dx, dy);
                }
            }
            if pressed(Key::PageUp) {
                if shift {
                    self.doc.to_front(&self.sel);
                } else {
                    self.doc.raise(&self.sel);
                }
            }
            if pressed(Key::PageDown) {
                if shift {
                    self.doc.to_back(&self.sel);
                } else {
                    self.doc.lower(&self.sel);
                }
            }
            if let [id] = self.sel.as_slice()
                && plain(Key::Enter)
            {
                self.begin_edit(*id);
            }
        }
        if plain(Key::Escape) {
            if self.tool.is_some() {
                self.tool = None;
            } else {
                self.sel.clear();
            }
        }
        if ctx.input(|i| i.modifiers.is_none()) {
            let l = self.lang();
            for kind in Kind::ALL {
                if pressed(kind.key(l)) {
                    self.tool = if self.tool == Some(kind) { None } else { Some(kind) };
                }
            }
        }
    }

    /// Pencereye bırakılan `.pafta` dosyasını açar.
    fn dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if let Some(path) = dropped
            .iter()
            .map(|f| f.path().to_path_buf())
            .find(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("pafta")))
        {
            self.request(Pending::OpenPath(path));
        }
    }

    fn preview_window(&mut self, ctx: &egui::Context) {
        if !self.preview_open {
            return;
        }
        let title = format!("{} — {}", self.doc.title, self.lang().pick("preview", "önizleme"));
        let builder = egui::ViewportBuilder::default()
            .with_title(title)
            .with_inner_size([self.doc.width, self.doc.height])
            .with_resizable(false);
        let doc = &self.doc;
        let live = &mut self.live;
        let mut close = false;
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("pafta-onizleme"),
            builder,
            |ui, _class| {
                live.render(ui, doc);
                if ui.input(|i| i.viewport().close_requested() || i.key_pressed(Key::Escape)) {
                    close = true;
                }
            },
        );
        if close {
            self.preview_open = false;
        }
    }

    fn confirm_dialog(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.pending.clone() else {
            return;
        };
        #[derive(Clone, Copy)]
        enum Choice {
            Save,
            Discard,
            Cancel,
        }
        let t = theme::tokens(ctx);
        let l = self.lang();
        let name = self.file_label();
        let mut choice = None;
        let modal = egui::Modal::new(Id::unique("kaydet-sor")).show(ctx, |ui| {
            ui.set_width(340.0);
            let title = l.pick("Save changes?", "Değişiklikler kaydedilsin mi?");
            ui.label(RichText::new(title).font(theme::semibold(13.0)).color(t.text));
            ui.add_space(4.0);
            let body = l.pick(
                format!("{name} hasn't been saved. Your recent changes will be lost if you don't save."),
                format!("{name} kaydedilmedi. Kaydetmezsen son değişiklikler kaybolur."),
            );
            ui.label(RichText::new(body).color(t.text2));
            ui.add_space(20.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if primary_button(ui, &t, l.pick("Save", "Kaydet")).clicked() {
                    choice = Some(Choice::Save);
                }
                if ui.button(l.pick("Don't Save", "Kaydetme")).clicked() {
                    choice = Some(Choice::Discard);
                }
                if ui.button(l.pick("Cancel", "Vazgeç")).clicked() {
                    choice = Some(Choice::Cancel);
                }
            });
        });
        if choice.is_none() && modal.should_close() {
            choice = Some(Choice::Cancel);
        }
        match choice {
            Some(Choice::Save) => {
                self.pending = None;
                if self.save() {
                    self.run(pending);
                }
            }
            Some(Choice::Discard) => {
                self.pending = None;
                self.run(pending);
            }
            Some(Choice::Cancel) => self.pending = None,
            None => {}
        }
    }

    fn help_dialog(&mut self, ctx: &egui::Context) {
        if !self.help_open {
            return;
        }
        let t = theme::tokens(ctx);
        let l = self.lang();
        let modal = egui::Modal::new(Id::unique("kisayollar")).show(ctx, |ui| {
            ui.set_width(480.0);
            let title = l.pick("Shortcuts", "Kısayollar");
            ui.label(RichText::new(title).font(theme::display(20.0)).color(t.text));
            ui.add_space(12.0);
            egui::Grid::new("kisayol-tablosu")
                .num_columns(2)
                .spacing(vec2(24.0, 8.0))
                .show(ui, |ui| {
                    for (what, keys) in shortcut_rows(l) {
                        ui.label(RichText::new(what).color(t.text));
                        ui.label(RichText::new(keys).font(theme::mono(12.0)).color(t.text2));
                        ui.end_row();
                    }
                });
            ui.add_space(16.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.button(l.pick("Close", "Kapat")).clicked()
            })
            .inner
        });
        if modal.inner || modal.should_close() {
            self.help_open = false;
        }
    }

    fn paint_toast(&mut self, ctx: &egui::Context) {
        let Some(toast) = &self.toast else {
            return;
        };
        if self.now >= toast.until {
            self.toast = None;
            return;
        }
        ctx.request_repaint_after(Duration::from_secs_f64(toast.until - self.now));
        let t = theme::tokens(ctx);
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::unique("bildirim")));
        let text_color = Color32::from_rgb(0xf5, 0xf5, 0xf7);
        let galley = painter.layout(toast.text.clone(), theme::body(13.0), text_color, 460.0);
        let pad = vec2(14.0, 8.0);
        let dot = if toast.error { 14.0 } else { 0.0 };
        let size = galley.size() + pad * 2.0 + vec2(dot, 0.0);
        let anchor = self.view.rect.center_bottom() - vec2(0.0, 84.0);
        let rect = Align2::CENTER_BOTTOM.anchor_size(anchor, size);
        let fill = if t.dark { Color32::from_rgb(0x3a, 0x3a, 0x3c) } else { t.text };
        painter.rect_filled(rect, 10.0, fill);
        if toast.error {
            let c = egui::pos2(rect.left() + pad.x + 4.0, rect.center().y);
            painter.circle_filled(c, 4.0, Color32::from_rgb(0xff, 0x45, 0x3a));
        }
        painter.galley(rect.min + pad + vec2(dot, 0.0), galley, text_color);
    }

    fn sync_title(&mut self, ctx: &egui::Context) {
        let dirty = if self.is_dirty() { " •" } else { "" };
        let title = format!("{}{dirty} — Pafta", self.file_label());
        if title != self.window_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.window_title = title;
        }
    }

    /// Fare bırakılmış ve yazı alanı odakta değilse değişiklikleri tek adım olarak kaydeder.
    fn end_frame(&mut self, ctx: &egui::Context) {
        self.doc.fix_radio_groups();
        let doc = &self.doc;
        self.sel.retain(|id| doc.get(*id).is_some());
        let busy = ctx.input(|i| i.pointer.any_down()) || ctx.text_edit_focused() || self.view.edit.is_some();
        if !busy {
            self.history.commit(&self.doc);
        }
    }
}

fn read_doc(path: &Path, l: Lang) -> Result<Document, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut doc: Document = serde_json::from_str(&text).map_err(|e| {
        l.pick(
            format!("not a valid Pafta file ({e})"),
            format!("geçerli bir Pafta dosyası değil ({e})"),
        )
    })?;
    doc.normalize();
    Ok(doc)
}

impl eframe::App for Pafta {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.now = ctx.input(|i| i.time);
        let t = theme::tokens(&ctx);
        self.shortcuts(&ctx);
        self.dropped_files(&ctx);

        let bar = egui::Frame::new().fill(t.surface).inner_margin(Margin::symmetric(12, 0));
        egui::Panel::top("ust")
            .exact_size(44.0)
            .frame(bar)
            .show(ui, |ui| self.top_bar(ui));
        let side = egui::Frame::new().fill(t.surface).inner_margin(Margin::symmetric(12, 16));
        egui::Panel::left("sol")
            .exact_size(216.0)
            .resizable(false)
            .frame(side)
            .show(ui, |ui| self.left_panel(ui));
        egui::Panel::right("sag")
            .exact_size(296.0)
            .resizable(false)
            .frame(side.inner_margin(Margin::symmetric(16, 16)))
            .show(ui, |ui| self.inspector(ui));
        if self.prefs.code_open {
            let code = egui::Frame::new().fill(t.surface).inner_margin(Margin {
                left: 16,
                right: 8,
                top: 10,
                bottom: 0,
            });
            egui::Panel::bottom("kod")
                .resizable(true)
                .default_size(260.0)
                .size_range(140.0..=640.0)
                .frame(code)
                .show(ui, |ui| self.code_panel(ui));
        }
        egui::CentralPanel::no_frame().show(ui, |ui| self.canvas_ui(ui));

        self.preview_window(&ctx);
        self.confirm_dialog(&ctx);
        self.help_dialog(&ctx);
        self.paint_toast(&ctx);
        self.sync_title(&ctx);
        self.end_frame(&ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let session = Session {
            doc: self.doc.clone(),
            saved: self.saved.clone(),
            path: self.path.clone(),
            prefs: self.prefs.clone(),
        };
        eframe::set_value(storage, eframe::APP_KEY, &session);
    }
}
