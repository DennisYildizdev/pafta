//! Tuval: pencerenin paftası ve elle yerleştirme (seç, taşı, boyutlandır, çiz).

use std::sync::Arc;

use eframe::egui::{
    self, Align2, Color32, CornerRadius, CursorIcon, FontId, Key, Painter, PointerButton, Pos2,
    Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, UiBuilder, Vec2, pos2, vec2,
};

use crate::app::Pafta;
use crate::i18n::Lang;
use crate::model::{Document, HAlign, Kind, MIN_SIDE, MIN_WINDOW, Widget};
use crate::panels::caps_galley;
use crate::snap::{self, Guide, Measure, Targets};
use crate::theme::{self, Tokens};

/// Çizilen başlık çubuğunun yüksekliği (tasarım pikseli).
const TITLE_BAR: f32 = 32.0;
/// Tutamaçların ekrandaki kenarı.
const HANDLE: f32 = 8.0;
/// Hizalama çizgisine çekilme mesafesi (ekran pikseli).
const SNAP_PX: f32 = 6.0;
const ZOOM_STEPS: [f32; 16] = [
    0.25, 0.33, 0.5, 0.67, 0.75, 0.8, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0, 4.0,
];

/// Paftanın mürekkebi. Pencerenin içi temadan bağımsız, açık ve tek renkli çizilir;
/// böylece seçim, hizalama ve ölçü işaretleri üstünde öne çıkar.
mod ink {
    use eframe::egui::Color32;

    pub const PAPER: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
    pub const BAR: Color32 = Color32::from_rgb(0xf3, 0xf3, 0xf5);
    pub const INK: Color32 = Color32::from_rgb(0x1d, 0x1d, 0x1f);
    pub const INK2: Color32 = Color32::from_rgb(0x6e, 0x6e, 0x73);
    pub const LINE: Color32 = Color32::from_rgb(0xc7, 0xc7, 0xcc);
    pub const FILL: Color32 = Color32::from_rgb(0xf2, 0xf2, 0xf4);
    pub const TRACK: Color32 = Color32::from_rgb(0xe5, 0xe5, 0xea);
    pub const GRID: Color32 = Color32::from_rgb(0xd5, 0xd5, 0xdb);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

impl Handle {
    const ALL: [Handle; 8] = [
        Handle::NW,
        Handle::N,
        Handle::NE,
        Handle::E,
        Handle::SE,
        Handle::S,
        Handle::SW,
        Handle::W,
    ];

    fn pos(self, r: Rect) -> Pos2 {
        match self {
            Handle::N => r.center_top(),
            Handle::S => r.center_bottom(),
            Handle::E => r.right_center(),
            Handle::W => r.left_center(),
            Handle::NE => r.right_top(),
            Handle::NW => r.left_top(),
            Handle::SE => r.right_bottom(),
            Handle::SW => r.left_bottom(),
        }
    }

    fn cursor(self) -> CursorIcon {
        match self {
            Handle::N | Handle::S => CursorIcon::ResizeVertical,
            Handle::E | Handle::W => CursorIcon::ResizeHorizontal,
            Handle::NE | Handle::SW => CursorIcon::ResizeNeSw,
            Handle::NW | Handle::SE => CursorIcon::ResizeNwSe,
        }
    }

    /// Hareket eden kenarlar: sol, üst, sağ, alt.
    fn edges(self) -> [bool; 4] {
        match self {
            Handle::N => [false, true, false, false],
            Handle::S => [false, false, false, true],
            Handle::E => [false, false, true, false],
            Handle::W => [true, false, false, false],
            Handle::NE => [false, true, true, false],
            Handle::NW => [true, true, false, false],
            Handle::SE => [false, false, true, true],
            Handle::SW => [true, false, false, true],
        }
    }
}

/// Küçük bileşenlerde kenar ortası tutamaçları köşelerle çakışmasın diye gizlenir.
fn visible_handles(r: Rect) -> impl Iterator<Item = Handle> {
    Handle::ALL.into_iter().filter(move |h| match h {
        Handle::N | Handle::S => r.width() >= 24.0,
        Handle::E | Handle::W => r.height() >= 24.0,
        _ => true,
    })
}

/// İmlecin altındaki şey.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hit {
    Handle(u64, Handle),
    Widget(u64),
    /// Pencerenin sağ ya da alt kenarı.
    Window(Handle),
    Empty,
}

#[derive(Clone, Debug, Default)]
pub enum Drag {
    #[default]
    None,
    Move {
        origin: Pos2,
        start: Vec<(u64, Rect)>,
    },
    Resize {
        id: u64,
        handle: Handle,
        origin: Pos2,
        start: Rect,
    },
    /// Seçili araçla yeni bir bileşen çiziliyor.
    Create {
        kind: Kind,
        origin: Pos2,
        rect: Rect,
    },
    Marquee {
        origin: Pos2,
        rect: Rect,
        base: Vec<u64>,
    },
    Window {
        handle: Handle,
        origin: Pos2,
        start: Vec2,
    },
    Pan,
}

/// Tuvalde, bileşenin üstünde metin düzenleme.
pub struct InlineEdit {
    pub id: u64,
    pub text: String,
    focus: bool,
}

pub struct View {
    pub zoom: f32,
    pub pan: Vec2,
    /// Bir sonraki karede pencereyi tuvale sığdır.
    pub fit: bool,
    /// Son karedeki tuval alanı.
    pub rect: Rect,
    pub drag: Drag,
    pub hover: Option<u64>,
    /// Katmanlar listesinde üstüne gelinen bileşen.
    pub layer_hover: Option<u64>,
    /// İmlecin pencere içindeki konumu.
    pub pointer: Option<Pos2>,
    pub edit: Option<InlineEdit>,
    pub(crate) guides: Vec<Guide>,
    press: Option<Hit>,
    /// Basışta seçime eklendi; bırakınca geri çıkarılmasın.
    press_added: bool,
    zoom_acc: f32,
}

impl Default for View {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            fit: true,
            rect: Rect::NOTHING,
            drag: Drag::None,
            hover: None,
            layer_hover: None,
            pointer: None,
            edit: None,
            guides: Vec::new(),
            press: None,
            press_added: false,
            zoom_acc: 1.0,
        }
    }
}

/// Tasarım pikselinden ekrana dönüşüm.
#[derive(Clone, Copy)]
struct Xf {
    origin: Pos2,
    zoom: f32,
}

impl Xf {
    fn new(canvas: Rect, doc: &Document, zoom: f32, pan: Vec2) -> Self {
        let size = vec2(doc.width, doc.height + TITLE_BAR) * zoom;
        let top_left = canvas.center() + pan - size / 2.0;
        Self {
            origin: (top_left + vec2(0.0, TITLE_BAR * zoom)).round(),
            zoom,
        }
    }

    fn pos(self, p: Pos2) -> Pos2 {
        self.origin + p.to_vec2() * self.zoom
    }

    fn rect(self, r: Rect) -> Rect {
        Rect::from_min_max(self.pos(r.min), self.pos(r.max))
    }

    fn inv(self, s: Pos2) -> Pos2 {
        pos2((s.x - self.origin.x) / self.zoom, (s.y - self.origin.y) / self.zoom)
    }
}

impl Pafta {
    /// Tasarım noktasının ekrandaki yeri (testler için).
    #[cfg(test)]
    pub(crate) fn to_screen(&self, p: Pos2) -> Pos2 {
        Xf::new(self.view.rect, &self.doc, self.view.zoom, self.view.pan).pos(p)
    }

    pub(crate) fn canvas_ui(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.ctx());
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let name = self.lang().pick("Canvas", "Tuval");
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Canvas, true, name));
        self.view.rect = rect;
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, t.canvas);

        if self.view.fit {
            self.fit_view();
        }
        self.navigate(ui, &response);
        let xf = Xf::new(rect, &self.doc, self.view.zoom, self.view.pan);
        self.interact(ui, &response, xf);

        let xf = Xf::new(rect, &self.doc, self.view.zoom, self.view.pan);
        let grid = self.prefs.show_grid.then_some(self.prefs.grid);
        paint_paper(&painter, xf, &self.doc, grid);
        let client = xf.rect(self.doc.client_rect());
        let paper = painter.with_clip_rect(client);
        let editing = self.view.edit.as_ref().map(|e| e.id);
        for w in &self.doc.widgets {
            paint_widget(&paper, xf, w, editing == Some(w.id));
        }
        if self.doc.widgets.is_empty() {
            paint_empty_hint(&paper, client, self.lang());
        }
        self.paint_overlays(&painter, ui, xf, &t);
        self.inline_edit_ui(ui, xf);
        self.title_block(ui, rect, &t);
    }

    pub(crate) fn fit_view(&mut self) {
        let avail = self.view.rect.shrink(48.0);
        if avail.width() <= 0.0 || avail.height() <= 0.0 {
            return;
        }
        let fit = (avail.width() / self.doc.width).min(avail.height() / (self.doc.height + TITLE_BAR));
        self.view.zoom = ZOOM_STEPS
            .iter()
            .rev()
            .copied()
            .find(|z| *z <= fit.min(1.0))
            .unwrap_or(ZOOM_STEPS[0]);
        self.view.pan = Vec2::ZERO;
        self.view.fit = false;
    }

    /// Bir adım yakınlaştırır (`step > 0`) ya da uzaklaştırır.
    pub(crate) fn zoom_step(&mut self, step: i32, anchor: Option<Pos2>) {
        let z = self.view.zoom;
        let next = if step > 0 {
            ZOOM_STEPS.iter().copied().find(|s| *s > z + 1e-3)
        } else {
            ZOOM_STEPS.iter().rev().copied().find(|s| *s < z - 1e-3)
        };
        if let Some(zoom) = next {
            self.set_zoom(zoom, anchor);
        }
    }

    pub(crate) fn zoom_reset(&mut self) {
        self.view.zoom = 1.0;
        self.view.pan = Vec2::ZERO;
    }

    /// Yakınlaştırırken `anchor` altındaki nokta yerinde kalır.
    fn set_zoom(&mut self, zoom: f32, anchor: Option<Pos2>) {
        let rect = self.view.rect;
        let anchor = anchor.filter(|p| rect.contains(*p)).unwrap_or(rect.center());
        let before = Xf::new(rect, &self.doc, self.view.zoom, self.view.pan);
        let design = before.inv(anchor);
        let after = Xf::new(rect, &self.doc, zoom, self.view.pan);
        self.view.pan += anchor - after.pos(design);
        self.view.zoom = zoom;
    }

    /// Tekerlek kaydırır, Ctrl + tekerlek yakınlaştırır, orta tuş sürükler.
    fn navigate(&mut self, ui: &Ui, response: &Response) {
        if response.hovered() {
            let (zoom, scroll, pointer) =
                ui.input(|i| (i.zoom_delta(), i.smooth_scroll_delta, i.pointer.hover_pos()));
            if zoom != 1.0 {
                self.view.zoom_acc *= zoom;
                let step = if self.view.zoom_acc > 1.08 {
                    1
                } else if self.view.zoom_acc < 1.0 / 1.08 {
                    -1
                } else {
                    0
                };
                if step != 0 {
                    self.view.zoom_acc = 1.0;
                    self.zoom_step(step, pointer);
                }
            }
            if scroll != Vec2::ZERO {
                self.view.pan += scroll;
            }
        }
        if response.dragged_by(PointerButton::Middle) {
            self.view.pan += response.drag_delta();
        }
    }

    fn interact(&mut self, ui: &Ui, response: &Response, xf: Xf) {
        let rect = self.view.rect;
        let (mods, space_down, pressed, released, press_origin, latest) = ui.input(|i| {
            (
                i.modifiers,
                i.key_down(Key::Space),
                i.pointer.primary_pressed(),
                i.pointer.primary_released(),
                i.pointer.press_origin(),
                i.pointer.latest_pos(),
            )
        });
        let space = space_down && !ui.ctx().text_edit_focused();
        let additive = mods.shift || mods.command;

        // Paletten sürüklenen bileşen bırakıldı.
        if let Some(kind) = self.palette_drag {
            if released {
                if let Some(p) = latest.filter(|p| rect.contains(*p)) {
                    let r = self.drop_rect(kind, xf.inv(p));
                    self.add_widget(kind, r);
                }
                self.palette_drag = None;
            }
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            return;
        }

        let hit = latest.filter(|_| response.hovered()).map(|p| self.hit(xf, p));
        self.view.hover = match hit {
            Some(Hit::Widget(id)) => Some(id),
            _ => None,
        };
        self.view.pointer = latest.filter(|p| rect.contains(*p)).map(|p| xf.inv(p));

        if pressed && response.hovered() && !space {
            let hit = hit.unwrap_or(Hit::Empty);
            self.view.press = Some(hit);
            self.view.press_added = false;
            if let (None, Hit::Widget(id)) = (self.tool, hit) {
                if additive {
                    if !self.sel.contains(&id) {
                        self.sel.push(id);
                        self.view.press_added = true;
                    }
                } else if !self.sel.contains(&id) {
                    self.sel = vec![id];
                }
            }
        }

        if response.drag_started_by(PointerButton::Primary) {
            let origin = xf.inv(press_origin.unwrap_or(rect.center()));
            self.view.drag = self.start_drag(origin, space, additive, mods.alt);
        }
        if response.dragged_by(PointerButton::Primary) {
            if matches!(self.view.drag, Drag::Pan) {
                self.view.pan += response.drag_delta();
            } else if let Some(p) = latest {
                self.drag_update(xf.inv(p), mods.alt, xf.zoom);
            }
        }
        if response.drag_stopped() {
            self.drag_end();
        }

        if response.double_clicked() {
            if let Some(Hit::Widget(id)) = self.view.press {
                self.begin_edit(id);
            }
        } else if response.clicked() && !space {
            self.click(latest.map(|p| xf.inv(p)), additive, mods.alt);
        }

        self.set_cursor(ui, response, hit, space);
    }

    fn hit(&self, xf: Xf, p: Pos2) -> Hit {
        if let [id] = self.sel.as_slice()
            && let Some(w) = self.doc.get(*id)
        {
            let r = xf.rect(w.rect());
            if let Some(h) = visible_handles(r).find(|h| h.pos(r).distance(p) <= HANDLE * 0.75) {
                return Hit::Handle(*id, h);
            }
        }
        let d = xf.inv(p);
        for w in self.doc.widgets.iter().rev() {
            let r = w.rect();
            let inside = if w.kind == Kind::Group && !self.sel.contains(&w.id) {
                // Seçili olmayan çerçeve yalnız kenarından ve başlığından tutulur;
                // içine basınca seçim kutusu çizilir.
                let band = 6.0 / xf.zoom;
                let on_edge = r.expand(band).contains(d) && !r.shrink(band).contains(d);
                let on_title = (d.y - r.top()).abs() <= 10.0 && d.x >= r.left() && d.x <= r.right();
                on_edge || on_title
            } else {
                r.expand(1.0 / xf.zoom).contains(d)
            };
            if inside {
                return Hit::Widget(w.id);
            }
        }
        let c = xf.rect(self.doc.client_rect());
        let near_right = (p.x - c.right()).abs() <= 5.0 && p.y >= c.top() && p.y <= c.bottom() + 5.0;
        let near_bottom = (p.y - c.bottom()).abs() <= 5.0 && p.x >= c.left() && p.x <= c.right() + 5.0;
        match (near_right, near_bottom) {
            (true, true) => Hit::Window(Handle::SE),
            (true, false) => Hit::Window(Handle::E),
            (false, true) => Hit::Window(Handle::S),
            (false, false) => Hit::Empty,
        }
    }

    fn start_drag(&self, origin: Pos2, space: bool, additive: bool, free: bool) -> Drag {
        if space {
            return Drag::Pan;
        }
        if let Some(kind) = self.tool {
            let o = self.snap_point(origin, free);
            return Drag::Create {
                kind,
                origin: o,
                rect: Rect::from_min_max(o, o),
            };
        }
        match self.view.press.unwrap_or(Hit::Empty) {
            Hit::Handle(id, handle) => self.doc.get(id).map_or(Drag::None, |w| Drag::Resize {
                id,
                handle,
                origin,
                start: w.rect(),
            }),
            Hit::Widget(_) => Drag::Move {
                origin,
                start: self
                    .move_set()
                    .into_iter()
                    .filter_map(|id| self.doc.get(id).map(|w| (id, w.rect())))
                    .collect(),
            },
            Hit::Window(handle) => Drag::Window {
                handle,
                origin,
                start: vec2(self.doc.width, self.doc.height),
            },
            Hit::Empty => Drag::Marquee {
                origin,
                rect: Rect::from_min_max(origin, origin),
                base: if additive { self.sel.clone() } else { Vec::new() },
            },
        }
    }

    /// `free`: Alt basılı, yaslama yok.
    fn drag_update(&mut self, cur: Pos2, free: bool, zoom: f32) {
        let threshold = SNAP_PX / zoom;
        let snapping = self.prefs.snap && !free;
        let grid = snapping.then_some(self.prefs.grid.max(1.0));
        let window = self.doc.client_rect();
        match self.view.drag.clone() {
            Drag::Move { origin, start } => {
                let ids: Vec<u64> = start.iter().map(|(id, _)| *id).collect();
                let Some(bbox) = start.iter().map(|(_, r)| *r).reduce(Rect::union) else {
                    return;
                };
                let mut delta = cur - origin;
                self.view.guides.clear();
                if snapping {
                    let others = self.doc.widgets.iter().filter(|w| !ids.contains(&w.id));
                    let targets = Targets::new(others.map(Widget::rect), window);
                    let snap = snap::snap_move(bbox.translate(delta), &targets, threshold, grid);
                    delta += snap.offset;
                    self.view.guides = snap.guides;
                }
                let delta = delta.round();
                for (id, r) in start {
                    if let Some(w) = self.doc.get_mut(id) {
                        w.set_rect(r.translate(delta));
                    }
                }
            }
            Drag::Resize { id, handle, origin, start } => {
                let d = cur - origin;
                let others = self.doc.widgets.iter().filter(|w| w.id != id);
                let targets = Targets::new(others.map(Widget::rect), window);
                let sx = |x: f32| {
                    if snapping { snap::snap_x(x, &targets, threshold, grid) } else { x.round() }
                };
                let sy = |y: f32| {
                    if snapping { snap::snap_y(y, &targets, threshold, grid) } else { y.round() }
                };
                let [left, top, right, bottom] = handle.edges();
                let mut r = start;
                if left {
                    r.min.x = sx(start.min.x + d.x).min(start.max.x - MIN_SIDE);
                }
                if right {
                    r.max.x = sx(start.max.x + d.x).max(start.min.x + MIN_SIDE);
                }
                if top {
                    r.min.y = sy(start.min.y + d.y).min(start.max.y - MIN_SIDE);
                }
                if bottom {
                    r.max.y = sy(start.max.y + d.y).max(start.min.y + MIN_SIDE);
                }
                self.view.guides = if snapping { snap::guides_for(r, &targets) } else { Vec::new() };
                if let Some(w) = self.doc.get_mut(id) {
                    w.set_rect(r);
                }
            }
            Drag::Create { kind, origin, .. } => {
                let p = self.snap_point(cur, free);
                self.view.drag = Drag::Create {
                    kind,
                    origin,
                    rect: Rect::from_two_pos(origin, p),
                };
            }
            Drag::Marquee { origin, base, .. } => {
                let m = Rect::from_two_pos(origin, cur);
                let mut sel = base.clone();
                for w in &self.doc.widgets {
                    let r = w.rect();
                    let caught = if w.kind == Kind::Group { m.contains_rect(r) } else { m.intersects(r) };
                    if caught && !sel.contains(&w.id) {
                        sel.push(w.id);
                    }
                }
                self.sel = sel;
                self.view.drag = Drag::Marquee { origin, rect: m, base };
            }
            Drag::Window { handle, origin, start } => {
                let [_, _, right, bottom] = handle.edges();
                let d = cur - origin;
                let fit = |v: f32| grid.map_or(v.round(), |g| (v / g).round() * g);
                let w = if right { fit(start.x + d.x).max(MIN_WINDOW.x) } else { self.doc.width };
                let h = if bottom { fit(start.y + d.y).max(MIN_WINDOW.y) } else { self.doc.height };
                // Pencere ortalanarak çizilir; sol üst köşe yerinde kalsın diye kaydırma düzeltilir.
                self.view.pan += vec2(w - self.doc.width, h - self.doc.height) * zoom / 2.0;
                self.doc.width = w;
                self.doc.height = h;
            }
            Drag::Pan | Drag::None => {}
        }
    }

    fn drag_end(&mut self) {
        if let Drag::Create { kind, rect, .. } = std::mem::take(&mut self.view.drag) {
            // Yalnız tıklanmış ya da tek yönde çekilmişse varsayılan boyut kullanılır.
            let size = kind.default_size();
            let w = if rect.width() >= 2.0 * MIN_SIDE { rect.width() } else { size.x };
            let h = if rect.height() >= 2.0 * MIN_SIDE { rect.height() } else { size.y };
            self.add_widget(kind, Rect::from_min_size(rect.min, vec2(w, h)));
        }
        self.view.guides.clear();
    }

    fn click(&mut self, at: Option<Pos2>, additive: bool, free: bool) {
        if let Some(kind) = self.tool {
            if let Some(p) = at {
                let min = self.snap_point(p, free);
                self.add_widget(kind, Rect::from_min_size(min, kind.default_size()));
            }
            return;
        }
        match self.view.press {
            Some(Hit::Widget(id)) => {
                if !additive {
                    self.sel = vec![id];
                } else if !self.view.press_added {
                    self.sel.retain(|s| *s != id);
                }
            }
            Some(Hit::Empty) | None => {
                if !additive {
                    self.sel.clear();
                }
            }
            Some(Hit::Handle(..) | Hit::Window(..)) => {}
        }
    }

    fn set_cursor(&self, ui: &Ui, response: &Response, hit: Option<Hit>, space: bool) {
        let icon = match &self.view.drag {
            Drag::Pan => Some(CursorIcon::Grabbing),
            Drag::Resize { handle, .. } | Drag::Window { handle, .. } => Some(handle.cursor()),
            Drag::Create { .. } => Some(CursorIcon::Crosshair),
            _ if response.dragged_by(PointerButton::Middle) => Some(CursorIcon::Grabbing),
            _ if !response.hovered() => None,
            _ if space => Some(CursorIcon::Grab),
            _ if self.tool.is_some() => Some(CursorIcon::Crosshair),
            _ => match hit {
                Some(Hit::Handle(_, h) | Hit::Window(h)) => Some(h.cursor()),
                _ => None,
            },
        };
        if let Some(icon) = icon {
            ui.ctx().set_cursor_icon(icon);
        }
    }

    pub(crate) fn add_widget(&mut self, kind: Kind, rect: Rect) {
        let id = self.doc.add(kind, rect, self.lang());
        self.sel = vec![id];
        self.tool = None;
    }

    /// Paletten bırakılan bileşen imlecin ortasına gelir.
    fn drop_rect(&self, kind: Kind, center: Pos2) -> Rect {
        let size = kind.default_size();
        Rect::from_min_size(self.snap_point(center - size / 2.0, false), size)
    }

    fn snap_point(&self, p: Pos2, free: bool) -> Pos2 {
        if free || !self.prefs.snap {
            return p.round();
        }
        let g = self.prefs.grid.max(1.0);
        pos2((p.x / g).round() * g, (p.y / g).round() * g)
    }

    /// Taşınacaklar: seçim ve seçili çerçevelerin içinde kalanlar.
    pub(crate) fn move_set(&self) -> Vec<u64> {
        let mut ids = self.sel.clone();
        for &id in &self.sel {
            for inner in self.doc.contents(id) {
                if !ids.contains(&inner) {
                    ids.push(inner);
                }
            }
        }
        ids
    }

    pub(crate) fn begin_edit(&mut self, id: u64) {
        let Some(w) = self.doc.get(id).filter(|w| w.kind.has_text()) else {
            return;
        };
        self.view.edit = Some(InlineEdit {
            id,
            text: w.text.clone(),
            focus: true,
        });
        self.sel = vec![id];
    }

    fn inline_edit_ui(&mut self, ui: &mut Ui, xf: Xf) {
        let Some(edit) = self.view.edit.as_mut() else {
            return;
        };
        let Some(w) = self.doc.get(edit.id) else {
            self.view.edit = None;
            return;
        };
        let area = xf.rect(text_area_of(w));
        let align = match (w.kind, w.align) {
            (Kind::Button, _) | (Kind::Label, HAlign::Center) => Align2::CENTER_CENTER,
            (Kind::Label, HAlign::Right) => Align2::RIGHT_CENTER,
            (Kind::TextArea, _) => Align2::LEFT_TOP,
            _ => Align2::LEFT_CENTER,
        };
        if w.kind == Kind::Group {
            ui.painter().rect_filled(area, 0.0, ink::PAPER);
        }
        let font = FontId::proportional(13.0 * xf.zoom);
        let editor = if w.kind == Kind::TextArea {
            egui::TextEdit::multiline(&mut edit.text)
        } else {
            egui::TextEdit::singleline(&mut edit.text)
        };
        let editor = editor
            .frame(egui::Frame::NONE)
            .font(font)
            .text_color(ink::INK)
            .align(align)
            .desired_width(area.width())
            .min_size(area.size());
        let style = paper_style(ui);
        let name = self.prefs.lang.pick("Component text", "Bileşenin metni");
        let response = ui.name_inputs(name, |ui| {
            ui.new_child(UiBuilder::new().max_rect(area).style(style))
                .add(editor)
        });
        if edit.focus {
            response.request_focus();
            edit.focus = false;
        }
        if response.lost_focus() {
            let cancelled = ui.input(|i| i.key_pressed(Key::Escape));
            if let Some(edit) = self.view.edit.take()
                && !cancelled
                && let Some(w) = self.doc.get_mut(edit.id)
            {
                w.text = edit.text;
            }
        }
    }

    fn paint_overlays(&self, painter: &Painter, ui: &Ui, xf: Xf, t: &Tokens) {
        let accent = t.accent;
        let client = xf.rect(self.doc.client_rect());

        for id in [self.view.hover, self.view.layer_hover].into_iter().flatten() {
            if self.sel.contains(&id) {
                continue;
            }
            if let Some(w) = self.doc.get(id) {
                let stroke = Stroke::new(1.0, t.family(w.kind.family()));
                painter.rect_stroke(xf.rect(w.rect()), 0.0, stroke, StrokeKind::Outside);
            }
        }

        let selected: Vec<&Widget> = self.sel.iter().filter_map(|id| self.doc.get(*id)).collect();

        // Ölçü çizgileri: sürüklerken ya da Alt basılıyken.
        let alt = ui.input(|i| i.modifiers.alt);
        let measured: Option<(Rect, Vec<u64>)> = match &self.view.drag {
            Drag::Move { start, .. } => {
                let ids: Vec<u64> = start.iter().map(|(id, _)| *id).collect();
                ids.iter()
                    .filter_map(|id| self.doc.get(*id))
                    .map(Widget::rect)
                    .reduce(Rect::union)
                    .map(|bbox| (bbox, ids))
            }
            Drag::Resize { id, .. } => self.doc.get(*id).map(|w| (w.rect(), vec![*id])),
            Drag::None if alt => match selected.as_slice() {
                [w] => Some((w.rect(), self.move_set())),
                _ => None,
            },
            _ => None,
        };

        for w in &selected {
            painter.rect_stroke(xf.rect(w.rect()), 0.0, Stroke::new(1.0, accent), StrokeKind::Outside);
        }
        match selected.as_slice() {
            [w] if self.view.edit.is_none() => {
                let r = xf.rect(w.rect());
                for h in visible_handles(r) {
                    let hr = Rect::from_center_size(h.pos(r), Vec2::splat(HANDLE));
                    painter.rect(hr, 1.0, Color32::WHITE, Stroke::new(1.0, accent), StrokeKind::Inside);
                }
                // Ölçüler görünürken boyut etiketi alttaki ölçüyle çakışmasın.
                if measured.is_none() {
                    let size = format!("{} × {}", w.w, w.h);
                    pill(painter, r.center_bottom() + vec2(0.0, 8.0), Align2::CENTER_TOP, &size, accent, t.on_accent);
                }
            }
            [] | [_] => {}
            many => {
                let bbox = many.iter().map(|w| xf.rect(w.rect())).reduce(Rect::union);
                if let Some(bbox) = bbox {
                    let stroke = Stroke::new(1.0, accent.gamma_multiply(0.5));
                    painter.rect_stroke(bbox.expand(4.0), 2.0, stroke, StrokeKind::Outside);
                }
            }
        }

        match &self.view.drag {
            Drag::Marquee { rect, .. } => {
                let fill = accent.gamma_multiply(0.08);
                painter.rect(xf.rect(*rect), 0.0, fill, Stroke::new(1.0, accent), StrokeKind::Inside);
            }
            Drag::Create { kind, rect, .. } if rect.width() >= 1.0 || rect.height() >= 1.0 => {
                let ghost = Widget::new(*kind, *rect, self.lang());
                paint_widget(&painter.with_clip_rect(client), xf, &ghost, false);
                let r = xf.rect(*rect);
                painter.rect_stroke(r, 0.0, Stroke::new(1.0, accent), StrokeKind::Outside);
                let size = format!("{} × {}", rect.width(), rect.height());
                pill(painter, r.center_bottom() + vec2(0.0, 8.0), Align2::CENTER_TOP, &size, accent, t.on_accent);
            }
            Drag::Window { .. } => {
                let size = format!("{} × {}", self.doc.width, self.doc.height);
                pill(painter, client.right_bottom() + vec2(8.0, 8.0), Align2::LEFT_TOP, &size, accent, t.on_accent);
            }
            _ => {}
        }

        let guide = Stroke::new(1.0, t.guide);
        for g in &self.view.guides {
            match *g {
                Guide::Vertical { x, y0, y1 } => {
                    let (a, b) = (xf.pos(pos2(x, y0)), xf.pos(pos2(x, y1)));
                    painter.vline(painter.round_to_pixel_center(a.x), a.y..=b.y, guide);
                }
                Guide::Horizontal { y, x0, x1 } => {
                    let (a, b) = (xf.pos(pos2(x0, y)), xf.pos(pos2(x1, y)));
                    painter.hline(a.x..=b.x, painter.round_to_pixel_center(a.y), guide);
                }
            }
        }

        if let Some((target, ids)) = measured {
            let others: Vec<Rect> = self
                .doc
                .widgets
                .iter()
                .filter(|w| !ids.contains(&w.id))
                .map(Widget::rect)
                .collect();
            let list = snap::measures(target, &others, self.doc.client_rect());
            paint_measures(painter, xf, &list, t.guide);
        }

        if let (Some(kind), Some(p)) = (self.palette_drag, ui.input(|i| i.pointer.latest_pos()))
            && self.view.rect.contains(p)
        {
            let r = self.drop_rect(kind, xf.inv(p));
            let mut ghost = painter.with_clip_rect(client);
            ghost.set_opacity(0.85);
            paint_widget(&ghost, xf, &Widget::new(kind, r, self.lang()), false);
            painter.rect_stroke(xf.rect(r), 0.0, Stroke::new(1.0, accent), StrokeKind::Outside);
        }
    }

    /// Sağ alttaki antet: pencere boyutu, imleç konumu ve ölçek.
    fn title_block(&mut self, ui: &mut Ui, rect: Rect, t: &Tokens) {
        let pointer = self.view.pointer.map_or_else(
            || "–".to_owned(),
            |p| format!("{}, {}", p.x.round(), p.y.round()),
        );
        let l = self.lang();
        let percent = (self.view.zoom * 100.0).round();
        // Türkçede yüzde işareti sayının önüne gelir.
        let zoom = l.pick(format!("{percent}%"), format!("%{percent}"));
        let cells = [
            (l.pick("SIZE", "BOYUT"), format!("{} × {}", self.doc.width, self.doc.height), "0000 × 0000"),
            (l.pick("POINTER", "İMLEÇ"), pointer, "-0000, -0000"),
            (l.pick("ZOOM", "ÖLÇEK"), zoom, "000%"),
        ];
        let painter = ui.painter_at(rect);
        let pad = vec2(12.0, 7.0);
        let label_h = 14.0;
        let value_font = theme::body(13.0);
        let widths: Vec<f32> = cells
            .iter()
            .map(|(label, _, template)| {
                let label_w = caps_galley(&painter, label, t.text2).size().x;
                let value_w = painter
                    .layout_no_wrap((*template).to_owned(), value_font.clone(), t.text)
                    .size()
                    .x;
                label_w.max(value_w) + pad.x * 2.0
            })
            .collect();
        let size = vec2(widths.iter().sum(), pad.y * 2.0 + label_h + 18.0);
        let block = Rect::from_min_size(rect.right_bottom() - size - vec2(16.0, 16.0), size);
        painter.rect(block, 8.0, t.surface, Stroke::new(1.0, t.separator), StrokeKind::Inside);

        let mut x = block.left();
        for (i, ((label, value, _), w)) in cells.iter().zip(&widths).enumerate() {
            let cell = Rect::from_min_size(pos2(x, block.top()), vec2(*w, block.height()));
            if i > 0 {
                painter.vline(painter.round_to_pixel_center(x), block.y_range(), Stroke::new(1.0, t.separator));
            }
            if i == cells.len() - 1 {
                let response = ui
                    .interact(cell, ui.scope_id().with("olcek"), Sense::click())
                    .on_hover_text(l.pick("Back to 100% (Ctrl+0)", "%100'e dön (Ctrl+0)"));
                let name = l.pick("Zoom: back to 100%", "Ölçek: %100'e dön");
                response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name));
                if response.hovered() {
                    painter.rect_filled(cell.shrink(3.0), 6.0, t.hover);
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                }
                if response.clicked() {
                    self.zoom_reset();
                }
            }
            painter.galley(cell.min + pad, caps_galley(&painter, label, t.text2), t.text2);
            painter.text(cell.min + pad + vec2(0.0, label_h), Align2::LEFT_TOP, value, value_font.clone(), t.text);
            x += w;
        }
    }
}

/// Satır içi düzenlemede metin imleci kâğıdın üstünde görünsün diye açık tema.
fn paper_style(ui: &Ui) -> Arc<egui::Style> {
    let mut style = (**ui.style()).clone();
    style.visuals = egui::Visuals::light();
    Arc::new(style)
}

/// Bileşenin metninin durduğu alan (tasarım pikseli).
fn text_area_of(w: &Widget) -> Rect {
    let r = w.rect();
    match w.kind {
        Kind::Entry => r.shrink2(vec2(8.0, 0.0)),
        Kind::TextArea => r.shrink2(vec2(6.0, 5.0)),
        Kind::Checkbox | Kind::Radio => Rect::from_min_max(pos2(r.left() + 22.0, r.top()), r.max),
        Kind::Group => Rect::from_min_size(
            pos2(r.left() + 6.0, r.top() - 10.0),
            vec2((r.width() - 12.0).max(40.0), 20.0),
        ),
        _ => r,
    }
}

fn paint_paper(painter: &Painter, xf: Xf, doc: &Document, grid: Option<f32>) {
    let z = xf.zoom;
    let client = xf.rect(doc.client_rect());
    let frame = Rect::from_min_max(client.min - vec2(0.0, TITLE_BAR * z), client.max);
    let radius = (8.0 * z).round().clamp(2.0, 12.0) as u8;
    let shadow = egui::Shadow {
        offset: [0, 2],
        blur: 18,
        spread: 0,
        color: Color32::from_black_alpha(26),
    };
    painter.add(shadow.as_shape(frame, CornerRadius::same(radius)));
    painter.rect(
        frame,
        CornerRadius::same(radius),
        ink::PAPER,
        Stroke::new(1.0, Color32::from_black_alpha(28)),
        StrokeKind::Outside,
    );
    let bar = Rect::from_min_max(frame.min, pos2(frame.max.x, client.min.y));
    let bar_radius = CornerRadius {
        nw: radius,
        ne: radius,
        sw: 0,
        se: 0,
    };
    painter.rect_filled(bar, bar_radius, ink::BAR);

    // Pencere düğmeleri, Windows'taki gibi 46 piksellik üç alan.
    let cell = 46.0 * z;
    let s = 5.0 * z;
    let glyph = Stroke::new(1.0, ink::INK2);
    let close = pos2(bar.right() - cell * 0.5, bar.center().y);
    painter.line_segment([close - vec2(s, s), close + vec2(s, s)], glyph);
    painter.line_segment([close + vec2(-s, s), close + vec2(s, -s)], glyph);
    let maximize = pos2(bar.right() - cell * 1.5, bar.center().y);
    painter.rect_stroke(Rect::from_center_size(maximize, Vec2::splat(2.0 * s)), 1.0, glyph, StrokeKind::Inside);
    let minimize = pos2(bar.right() - cell * 2.5, bar.center().y);
    painter.hline(minimize.x - s..=minimize.x + s, minimize.y, glyph);
    painter.with_clip_rect(bar.with_max_x(bar.right() - cell * 3.0)).text(
        pos2(bar.left() + 12.0 * z, bar.center().y),
        Align2::LEFT_CENTER,
        &doc.title,
        FontId::proportional(12.0 * z),
        ink::INK,
    );

    if let Some(g) = grid.filter(|g| g * z >= 6.0) {
        let mut dots = Vec::new();
        let mut y = g;
        while y < doc.height {
            let mut x = g;
            while x < doc.width {
                let p = xf.pos(pos2(x, y));
                dots.push(Shape::rect_filled(Rect::from_min_size(p, Vec2::splat(1.0)), 0.0, ink::GRID));
                x += g;
            }
            y += g;
        }
        painter.extend(dots);
    }
}

/// Bileşeni paftaya çizer. `hide_text`: üstünde metin düzenleniyor.
fn paint_widget(painter: &Painter, xf: Xf, w: &Widget, hide_text: bool) {
    let z = xf.zoom;
    let r = xf.rect(w.rect());
    let clip = if w.kind == Kind::Group {
        Rect::from_min_max(r.min - vec2(2.0, 12.0 * z), r.max + vec2(2.0, 2.0))
    } else {
        r.expand(2.0)
    };
    let p = painter.with_clip_rect(clip);
    let font = FontId::proportional(13.0 * z);
    let line = Stroke::new(1.0, ink::LINE);
    let px = |v: f32| CornerRadius::same((v * z).round().clamp(0.0, 255.0) as u8);
    let text = |pos: Pos2, anchor: Align2, s: &str| {
        if !hide_text && !s.is_empty() {
            p.text(pos, anchor, s, font.clone(), ink::INK);
        }
    };
    match w.kind {
        Kind::Label => {
            let (pos, anchor) = match w.align {
                HAlign::Left => (r.left_center(), Align2::LEFT_CENTER),
                HAlign::Center => (r.center(), Align2::CENTER_CENTER),
                HAlign::Right => (r.right_center(), Align2::RIGHT_CENTER),
            };
            text(pos, anchor, &w.text);
        }
        Kind::Button => {
            p.rect(r, px(5.0), ink::FILL, line, StrokeKind::Inside);
            text(r.center(), Align2::CENTER_CENTER, &w.text);
        }
        Kind::Entry => {
            p.rect(r, px(5.0), ink::PAPER, line, StrokeKind::Inside);
            text(r.left_center() + vec2(8.0 * z, 0.0), Align2::LEFT_CENTER, &w.text);
        }
        Kind::TextArea => {
            p.rect(r, px(5.0), ink::PAPER, line, StrokeKind::Inside);
            if !hide_text && !w.text.is_empty() {
                let wrap = (r.width() - 12.0 * z).max(1.0);
                let galley = p.layout(w.text.clone(), font.clone(), ink::INK, wrap);
                p.galley(r.left_top() + vec2(6.0, 5.0) * z, galley, ink::INK);
            }
        }
        Kind::Checkbox => {
            let b = Rect::from_center_size(pos2(r.left() + 8.0 * z, r.center().y), Vec2::splat(14.0 * z));
            if w.checked {
                p.rect_filled(b, px(3.0), ink::INK);
                let tick = vec![
                    b.lerp_inside(vec2(0.24, 0.52)),
                    b.lerp_inside(vec2(0.43, 0.70)),
                    b.lerp_inside(vec2(0.77, 0.32)),
                ];
                p.line(tick, Stroke::new(1.6 * z, ink::PAPER));
            } else {
                p.rect(b, px(3.0), ink::PAPER, Stroke::new(1.0, ink::INK2), StrokeKind::Inside);
            }
            text(pos2(b.right() + 6.0 * z, r.center().y), Align2::LEFT_CENTER, &w.text);
        }
        Kind::Radio => {
            let c = pos2(r.left() + 8.0 * z, r.center().y);
            if w.checked {
                p.circle_filled(c, 7.0 * z, ink::INK);
                p.circle_filled(c, 2.8 * z, ink::PAPER);
            } else {
                p.circle(c, 7.0 * z - 0.5, ink::PAPER, Stroke::new(1.0, ink::INK2));
            }
            text(pos2(c.x + 13.0 * z, r.center().y), Align2::LEFT_CENTER, &w.text);
        }
        Kind::Combo => {
            p.rect(r, px(5.0), ink::FILL, line, StrokeKind::Inside);
            let items = w.clean_items();
            let shown = items.get(w.selected).copied().unwrap_or_default();
            text(r.left_center() + vec2(8.0 * z, 0.0), Align2::LEFT_CENTER, shown);
            let c = pos2(r.right() - 12.0 * z, r.center().y);
            let chevron = vec![c + vec2(-3.5, -1.5) * z, c + vec2(0.0, 2.0) * z, c + vec2(3.5, -1.5) * z];
            p.line(chevron, Stroke::new(1.3 * z, ink::INK2));
        }
        Kind::List => {
            p.rect(r, px(5.0), ink::PAPER, line, StrokeKind::Inside);
            if !hide_text {
                for (i, item) in w.clean_items().iter().enumerate() {
                    let y = r.top() + (14.0 + 20.0 * i as f32) * z;
                    if y > r.bottom() {
                        break;
                    }
                    p.text(pos2(r.left() + 8.0 * z, y), Align2::LEFT_CENTER, item, font.clone(), ink::INK);
                }
            }
        }
        Kind::Slider => {
            let span = (w.max - w.min).abs().max(f32::EPSILON);
            let t = ((w.value - w.min.min(w.max)) / span).clamp(0.0, 1.0);
            let knob = 7.0 * z;
            let (x0, x1) = (r.left() + knob, (r.right() - knob).max(r.left() + knob));
            let x = egui::lerp(x0..=x1, t);
            let track = Rect::from_min_max(pos2(x0, r.center().y - 2.0 * z), pos2(x1, r.center().y + 2.0 * z));
            p.rect_filled(track, px(2.0), ink::TRACK);
            p.rect_filled(Rect::from_min_max(track.min, pos2(x, track.max.y)), px(2.0), ink::INK2);
            p.circle(pos2(x, r.center().y), knob, ink::PAPER, Stroke::new(1.0, ink::INK2));
        }
        Kind::Progress => {
            let t = (w.value / 100.0).clamp(0.0, 1.0);
            let round = CornerRadius::same((r.height() / 2.0).min(255.0) as u8);
            p.rect_filled(r, round, ink::TRACK);
            if t > 0.0 {
                let done = Rect::from_min_max(r.min, pos2(egui::lerp(r.left()..=r.right(), t), r.max.y));
                p.rect_filled(done, round, ink::INK2);
            }
        }
        Kind::Group => {
            p.rect_stroke(r, px(6.0), line, StrokeKind::Inside);
            if !hide_text && !w.text.is_empty() {
                let galley = p.layout_no_wrap(w.text.clone(), font.clone(), ink::INK);
                let pos = r.left_top() + vec2(10.0 * z, -galley.size().y / 2.0);
                let gap = Rect::from_min_size(pos, galley.size()).expand2(vec2(4.0 * z, 0.0));
                p.rect_filled(gap, 0.0, ink::PAPER);
                p.galley(pos, galley, ink::INK);
            }
        }
    }
}

fn paint_empty_hint(painter: &Painter, client: Rect, l: Lang) {
    let c = client.center();
    let title = l.pick("Empty window", "Boş pencere");
    painter.text(c - vec2(0.0, 4.0), Align2::CENTER_BOTTOM, title, theme::semibold(13.0), ink::INK2);
    painter.text(
        c + vec2(0.0, 4.0),
        Align2::CENTER_TOP,
        l.pick(
            "Drag a component from the left, or press its letter.",
            "Soldan bir bileşen sürükle ya da harfine bas.",
        ),
        theme::body(13.0),
        ink::INK2,
    );
}

/// Küçük, dolu, yuvarlak köşeli etiket: ölçüler ve boyutlar için.
fn pill(painter: &Painter, pos: Pos2, anchor: Align2, text: &str, fill: Color32, color: Color32) {
    let galley = painter.layout_no_wrap(text.to_owned(), theme::body(11.0), color);
    let rect = anchor.anchor_size(pos, galley.size() + vec2(10.0, 4.0));
    painter.rect_filled(rect, 4.0, fill);
    painter.galley(rect.center() - galley.size() / 2.0, galley, color);
}

/// Teknik çizimdeki gibi uçları çentikli ölçü çizgileri.
fn paint_measures(painter: &Painter, xf: Xf, list: &[Measure], color: Color32) {
    let stroke = Stroke::new(1.0, color);
    for m in list {
        let (a, b) = (xf.pos(m.from), xf.pos(m.to));
        painter.line_segment([a, b], stroke);
        let tick = (b - a).normalized().rot90() * 4.0;
        painter.line_segment([a - tick, a + tick], stroke);
        painter.line_segment([b - tick, b + tick], stroke);
        // Sayı çizginin üstüne değil yanına yazılır; kısa ölçülerde çizgi görünür kalır.
        let label = format!("{}", m.value.round());
        let mid = a.lerp(b, 0.5);
        let (pos, anchor) = if (b - a).x.abs() > (b - a).y.abs() {
            (mid - vec2(0.0, 5.0), Align2::CENTER_BOTTOM)
        } else {
            (mid + vec2(6.0, 0.0), Align2::LEFT_CENTER)
        };
        pill(painter, pos, anchor, &label, color, Color32::WHITE);
    }
}
