//! Belge modeli: tasarlanan pencere ve üzerindeki bileşenler.

use std::collections::HashSet;

use eframe::egui::{self, Rect, Vec2, pos2, vec2};
use serde::{Deserialize, Serialize};

use crate::i18n::Lang;

/// Bir bileşenin en küçük kenarı (tasarım pikseli).
pub const MIN_SIDE: f32 = 4.0;

/// Pencerenin en küçük iç boyutu.
pub const MIN_WINDOW: Vec2 = vec2(160.0, 96.0);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Kind {
    #[default]
    Label,
    Button,
    Entry,
    TextArea,
    Checkbox,
    Radio,
    Combo,
    List,
    Slider,
    Progress,
    Group,
}

/// Paletteki kümeler. Her kümenin bir kimlik rengi var.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Text,
    Action,
    Input,
    Choice,
    Value,
    Layout,
}

impl Kind {
    pub const ALL: [Kind; 11] = [
        Kind::Label,
        Kind::Button,
        Kind::Entry,
        Kind::TextArea,
        Kind::Checkbox,
        Kind::Radio,
        Kind::Combo,
        Kind::List,
        Kind::Slider,
        Kind::Progress,
        Kind::Group,
    ];

    pub fn title(self, lang: Lang) -> &'static str {
        match self {
            Kind::Label => lang.pick("Label", "Etiket"),
            Kind::Button => lang.pick("Button", "Düğme"),
            Kind::Entry => lang.pick("Text field", "Giriş"),
            Kind::TextArea => lang.pick("Text area", "Metin alanı"),
            Kind::Checkbox => lang.pick("Checkbox", "Onay kutusu"),
            Kind::Radio => lang.pick("Radio button", "Seçenek"),
            Kind::Combo => lang.pick("Dropdown", "Açılır liste"),
            Kind::List => lang.pick("List", "Liste"),
            Kind::Slider => lang.pick("Slider", "Kaydırıcı"),
            Kind::Progress => lang.pick("Progress bar", "İlerleme"),
            Kind::Group => lang.pick("Group box", "Çerçeve"),
        }
    }

    /// Başlığın büyük harfli hâli. `to_uppercase` Türkçe İ'yi bilmediği için elle yazıldı.
    pub fn caption(self, lang: Lang) -> &'static str {
        match self {
            Kind::Label => lang.pick("LABEL", "ETİKET"),
            Kind::Button => lang.pick("BUTTON", "DÜĞME"),
            Kind::Entry => lang.pick("TEXT FIELD", "GİRİŞ"),
            Kind::TextArea => lang.pick("TEXT AREA", "METİN ALANI"),
            Kind::Checkbox => lang.pick("CHECKBOX", "ONAY KUTUSU"),
            Kind::Radio => lang.pick("RADIO BUTTON", "SEÇENEK"),
            Kind::Combo => lang.pick("DROPDOWN", "AÇILIR LİSTE"),
            Kind::List => lang.pick("LIST", "LİSTE"),
            Kind::Slider => lang.pick("SLIDER", "KAYDIRICI"),
            Kind::Progress => lang.pick("PROGRESS BAR", "İLERLEME"),
            Kind::Group => lang.pick("GROUP BOX", "ÇERÇEVE"),
        }
    }

    /// Paletteki kısayol tuşu: adın baş harfi, dile göre.
    pub fn key(self, lang: Lang) -> egui::Key {
        use egui::Key;
        match self {
            Kind::Label => lang.pick(Key::T, Key::E),
            Kind::Button => lang.pick(Key::B, Key::D),
            Kind::Entry => lang.pick(Key::F, Key::G),
            Kind::TextArea => lang.pick(Key::A, Key::M),
            Kind::Checkbox => lang.pick(Key::C, Key::O),
            Kind::Radio => lang.pick(Key::R, Key::S),
            Kind::Combo => lang.pick(Key::D, Key::A),
            Kind::List => Key::L,
            Kind::Slider => lang.pick(Key::S, Key::K),
            Kind::Progress => lang.pick(Key::P, Key::I),
            Kind::Group => lang.pick(Key::G, Key::C),
        }
    }

    /// Yeni bileşenlerin ad öneki: `button_1` ya da `dugme_1`...
    fn prefix(self, lang: Lang) -> &'static str {
        match self {
            Kind::Label => lang.pick("label", "etiket"),
            Kind::Button => lang.pick("button", "dugme"),
            Kind::Entry => lang.pick("field", "giris"),
            Kind::TextArea => lang.pick("text", "metin"),
            Kind::Checkbox => lang.pick("check", "onay"),
            Kind::Radio => lang.pick("radio", "secenek"),
            Kind::Combo => lang.pick("dropdown", "acilir"),
            Kind::List => lang.pick("list", "liste"),
            Kind::Slider => lang.pick("slider", "kaydirici"),
            Kind::Progress => lang.pick("progress", "ilerleme"),
            Kind::Group => lang.pick("group", "cerceve"),
        }
    }

    pub fn family(self) -> Family {
        match self {
            Kind::Label => Family::Text,
            Kind::Button => Family::Action,
            Kind::Entry | Kind::TextArea => Family::Input,
            Kind::Checkbox | Kind::Radio | Kind::Combo | Kind::List => Family::Choice,
            Kind::Slider | Kind::Progress => Family::Value,
            Kind::Group => Family::Layout,
        }
    }

    pub fn default_size(self) -> Vec2 {
        match self {
            Kind::Label => vec2(120.0, 20.0),
            Kind::Button => vec2(96.0, 28.0),
            Kind::Entry => vec2(184.0, 28.0),
            Kind::TextArea => vec2(240.0, 96.0),
            Kind::Checkbox => vec2(160.0, 22.0),
            Kind::Radio => vec2(140.0, 22.0),
            Kind::Combo => vec2(160.0, 28.0),
            Kind::List => vec2(160.0, 104.0),
            Kind::Slider => vec2(184.0, 24.0),
            Kind::Progress => vec2(184.0, 8.0),
            Kind::Group => vec2(240.0, 144.0),
        }
    }

    fn default_text(self, lang: Lang) -> &'static str {
        match self {
            Kind::Label => lang.pick("Label", "Etiket"),
            Kind::Button => lang.pick("Button", "Düğme"),
            Kind::Checkbox => lang.pick("Checkbox", "Onay kutusu"),
            Kind::Radio => lang.pick("Option", "Seçenek"),
            Kind::Group => lang.pick("Group", "Çerçeve"),
            _ => "",
        }
    }

    /// Tuvalde çift tıklayınca düzenlenen bir metni var mı?
    pub fn has_text(self) -> bool {
        matches!(
            self,
            Kind::Label
                | Kind::Button
                | Kind::Entry
                | Kind::TextArea
                | Kind::Checkbox
                | Kind::Radio
                | Kind::Group
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl HAlign {
    pub fn egui(self) -> egui::Align {
        match self {
            HAlign::Left => egui::Align::LEFT,
            HAlign::Center => egui::Align::Center,
            HAlign::Right => egui::Align::RIGHT,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Widget {
    pub id: u64,
    pub kind: Kind,
    /// Koddaki adı: alan ya da yöntem adı olur.
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub text: String,
    /// Açılır listenin ve listenin satırları.
    pub items: Vec<String>,
    /// Açılır listede seçili satır.
    pub selected: usize,
    /// Kaydırıcının ve ilerleme çubuğunun değeri (ilerleme için 0–100).
    pub value: f32,
    pub min: f32,
    pub max: f32,
    /// Onay kutusu işaretli ya da seçenek seçili.
    pub checked: bool,
    /// Seçeneğin grubu: aynı gruptakilerden yalnız biri seçili olur.
    pub group: String,
    pub align: HAlign,
}

impl Default for Widget {
    fn default() -> Self {
        Self {
            id: 0,
            kind: Kind::Label,
            name: String::new(),
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 24.0,
            text: String::new(),
            items: Vec::new(),
            selected: 0,
            value: 0.0,
            min: 0.0,
            max: 100.0,
            checked: false,
            group: "choice".to_owned(),
            align: HAlign::Left,
        }
    }
}

impl Widget {
    /// Yeni bir bileşenin varsayılan hâli. Kimliği ve adı belge verir.
    pub fn new(kind: Kind, rect: Rect, lang: Lang) -> Self {
        let mut widget = Self {
            kind,
            text: kind.default_text(lang).to_owned(),
            group: lang.pick("choice", "secim").to_owned(),
            ..Self::default()
        };
        match kind {
            Kind::Combo | Kind::List => {
                let items = lang.pick(["First", "Second", "Third"], ["Birinci", "İkinci", "Üçüncü"]);
                widget.items = items.map(String::from).to_vec();
            }
            Kind::Slider => widget.value = 50.0,
            Kind::Progress => widget.value = 40.0,
            _ => {}
        }
        widget.set_rect(rect);
        widget
    }

    pub fn rect(&self) -> Rect {
        Rect::from_min_size(pos2(self.x, self.y), vec2(self.w, self.h))
    }

    /// Konumu ve boyutu tam piksele yuvarlayarak yazar.
    pub fn set_rect(&mut self, r: Rect) {
        self.x = r.min.x.round();
        self.y = r.min.y.round();
        self.w = r.width().round().max(MIN_SIDE);
        self.h = r.height().round().max(MIN_SIDE);
    }

    /// Boş satırları atılmış öğeler.
    pub fn clean_items(&self) -> Vec<&str> {
        self.items
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignOp {
    Left,
    CenterX,
    Right,
    Top,
    CenterY,
    Bottom,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Document {
    pub title: String,
    /// Pencerenin iç alanı (başlık çubuğu hariç).
    pub width: f32,
    pub height: f32,
    /// Çizim sırasıyla: ilk bileşen en altta kalır.
    pub widgets: Vec<Widget>,
    pub next_id: u64,
}

impl Default for Document {
    fn default() -> Self {
        Self::new(Lang::En)
    }
}

impl Document {
    pub fn new(lang: Lang) -> Self {
        Self {
            title: lang.pick("New window", "Yeni pencere").to_owned(),
            width: 480.0,
            height: 320.0,
            widgets: Vec::new(),
            next_id: 1,
        }
    }

    /// İlk açılışta gösterilen tasarım: küçük bir görsel dönüştürücü.
    pub fn example(lang: Lang) -> Self {
        let l = lang;
        let mut doc = Self {
            title: l.pick("Image converter", "Görsel dönüştürücü").to_owned(),
            width: 440.0,
            height: 360.0,
            ..Self::new(lang)
        };
        let source = l.pick(r"D:\Photos\2026", r"D:\Fotoğraflar\2026");
        doc.put(Kind::Label, l.pick("source_label", "kaynak_etiket"), [24.0, 20.0, 200.0, 20.0], l.pick("Source folder", "Kaynak klasör"));
        doc.put(Kind::Entry, l.pick("source", "kaynak"), [24.0, 44.0, 288.0, 28.0], source);
        doc.put(Kind::Button, l.pick("browse", "gozat"), [320.0, 44.0, 96.0, 28.0], l.pick("Browse…", "Gözat…"));
        doc.put(Kind::Label, l.pick("format_label", "bicim_etiket"), [24.0, 88.0, 184.0, 20.0], l.pick("Format", "Biçim"));
        let format = doc.put(Kind::Combo, l.pick("format", "bicim"), [24.0, 112.0, 184.0, 28.0], "");
        format.items = ["PNG", "JPEG", "WebP"].map(String::from).to_vec();
        format.selected = 1;
        doc.put(Kind::Label, l.pick("quality_label", "kalite_etiket"), [232.0, 88.0, 184.0, 20.0], l.pick("Quality", "Kalite"));
        doc.put(Kind::Slider, l.pick("quality", "kalite"), [232.0, 112.0, 184.0, 28.0], "").value = 85.0;
        let subfolders = l.pick("Include subfolders", "Alt klasörleri de tara");
        doc.put(Kind::Checkbox, l.pick("include_subfolders", "alt_klasorler"), [24.0, 156.0, 240.0, 22.0], subfolders)
            .checked = true;
        doc.put(Kind::Group, l.pick("output_frame", "cikti_cercevesi"), [24.0, 196.0, 392.0, 64.0], l.pick("Output", "Çıktı"));
        for (name, x, w, text) in [
            (l.pick("same_folder", "ayni_klasor"), 40.0, 184.0, l.pick("Same folder as source", "Kaynakla aynı klasöre")),
            (l.pick("new_folder", "yeni_klasor"), 232.0, 168.0, l.pick("New folder", "Yeni klasöre")),
        ] {
            doc.put(Kind::Radio, name, [x, 216.0, w, 22.0], text).group = l.pick("output", "cikti").to_owned();
        }
        doc.put(Kind::Progress, l.pick("progress", "ilerleme"), [24.0, 280.0, 392.0, 8.0], "").value = 40.0;
        doc.put(Kind::Button, l.pick("cancel", "vazgec"), [216.0, 308.0, 96.0, 28.0], l.pick("Cancel", "Vazgeç"));
        doc.put(Kind::Button, l.pick("convert", "donustur"), [320.0, 308.0, 96.0, 28.0], l.pick("Convert", "Dönüştür"));
        doc.fix_radio_groups();
        doc
    }

    fn put(&mut self, kind: Kind, name: &str, [x, y, w, h]: [f32; 4], text: &str) -> &mut Widget {
        let mut widget = Widget::new(kind, Rect::from_min_size(pos2(x, y), vec2(w, h)), Lang::En);
        widget.id = self.alloc_id();
        widget.name = name.to_owned();
        widget.text = text.to_owned();
        self.widgets.push(widget);
        self.widgets.last_mut().expect("az önce eklendi")
    }

    pub fn client_rect(&self) -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), vec2(self.width, self.height))
    }

    pub fn get(&self, id: u64) -> Option<&Widget> {
        self.widgets.iter().find(|w| w.id == id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Widget> {
        self.widgets.iter_mut().find(|w| w.id == id)
    }

    pub fn index_of(&self, id: u64) -> Option<usize> {
        self.widgets.iter().position(|w| w.id == id)
    }

    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id.max(1);
        self.next_id = id + 1;
        id
    }

    /// `onek_1`, `onek_2`... içinden boştaki ilk adı verir.
    fn unique_name(&self, prefix: &str) -> String {
        (1..)
            .map(|n| format!("{prefix}_{n}"))
            .find(|name| !self.widgets.iter().any(|w| &w.name == name))
            .expect("sonsuz aralıkta boş ad var")
    }

    pub fn name_taken(&self, name: &str, except: u64) -> bool {
        self.widgets.iter().any(|w| w.id != except && w.name == name)
    }

    /// Yeni bir bileşen ekler ve kimliğini döndürür. Varsayılan ad ve metin `lang` dilindedir.
    pub fn add(&mut self, kind: Kind, rect: Rect, lang: Lang) -> u64 {
        let mut widget = Widget::new(kind, rect, lang);
        widget.id = self.alloc_id();
        widget.name = self.unique_name(kind.prefix(lang));
        let id = widget.id;
        self.widgets.push(widget);
        self.fix_radio_groups();
        id
    }

    pub fn remove(&mut self, ids: &[u64]) {
        self.widgets.retain(|w| !ids.contains(&w.id));
        self.fix_radio_groups();
    }

    /// Seçilenleri `offset` kadar kaydırılmış olarak kopyalar.
    pub fn duplicate(&mut self, ids: &[u64], offset: Vec2) -> Vec<u64> {
        let copies = self
            .widgets
            .iter()
            .filter(|w| ids.contains(&w.id))
            .cloned()
            .collect();
        self.insert_copies(copies, offset)
    }

    /// Kopyalanmış bileşenleri yeni kimlik ve gerekirse yeni adla ekler.
    pub fn insert_copies(&mut self, copies: Vec<Widget>, offset: Vec2) -> Vec<u64> {
        let mut ids = Vec::with_capacity(copies.len());
        for mut widget in copies {
            widget.id = self.alloc_id();
            if self.widgets.iter().any(|w| w.name == widget.name) {
                widget.name = self.unique_name(name_stem(&widget.name));
            }
            if widget.kind == Kind::Radio {
                widget.checked = false;
            }
            widget.x += offset.x;
            widget.y += offset.y;
            ids.push(widget.id);
            self.widgets.push(widget);
        }
        self.fix_radio_groups();
        ids
    }

    /// Seçeneği grubunda seçili yapar, gruptaki diğerlerini bırakır.
    pub fn select_radio(&mut self, id: u64) {
        let Some(group) = self.get(id).map(|w| w.group.clone()) else {
            return;
        };
        for w in &mut self.widgets {
            if w.kind == Kind::Radio && w.group == group {
                w.checked = w.id == id;
            }
        }
    }

    /// Her seçenek grubunda tam olarak bir seçili seçenek bırakır.
    pub fn fix_radio_groups(&mut self) {
        let mut groups: Vec<String> = Vec::new();
        for w in &self.widgets {
            if w.kind == Kind::Radio && !groups.contains(&w.group) {
                groups.push(w.group.clone());
            }
        }
        for group in groups {
            let in_group = |w: &&Widget| w.kind == Kind::Radio && w.group == group;
            let keep = self
                .widgets
                .iter()
                .filter(in_group)
                .find(|w| w.checked)
                .or_else(|| self.widgets.iter().find(in_group))
                .map(|w| w.id);
            for w in &mut self.widgets {
                if w.kind == Kind::Radio && w.group == group {
                    w.checked = Some(w.id) == keep;
                }
            }
        }
    }

    /// Gruptaki seçili seçeneğin sırası.
    pub fn radio_initial(&self, group: &str) -> usize {
        self.widgets
            .iter()
            .filter(|w| w.kind == Kind::Radio && w.group == group)
            .position(|w| w.checked)
            .unwrap_or(0)
    }

    /// Çerçevenin içinde kalan ve üstünde çizilen bileşenler.
    pub fn contents(&self, group: u64) -> Vec<u64> {
        let Some(index) = self.index_of(group) else {
            return Vec::new();
        };
        let frame = &self.widgets[index];
        if frame.kind != Kind::Group {
            return Vec::new();
        }
        let outer = frame.rect();
        self.widgets[index + 1..]
            .iter()
            .filter(|w| outer.contains_rect(w.rect()))
            .map(|w| w.id)
            .collect()
    }

    /// Bir adım öne alır.
    pub fn raise(&mut self, ids: &[u64]) {
        for i in (0..self.widgets.len().saturating_sub(1)).rev() {
            if ids.contains(&self.widgets[i].id) && !ids.contains(&self.widgets[i + 1].id) {
                self.widgets.swap(i, i + 1);
            }
        }
    }

    /// Bir adım arkaya alır.
    pub fn lower(&mut self, ids: &[u64]) {
        for i in 1..self.widgets.len() {
            if ids.contains(&self.widgets[i].id) && !ids.contains(&self.widgets[i - 1].id) {
                self.widgets.swap(i, i - 1);
            }
        }
    }

    pub fn to_front(&mut self, ids: &[u64]) {
        let (picked, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.widgets)
            .into_iter()
            .partition(|w| ids.contains(&w.id));
        self.widgets = rest;
        self.widgets.extend(picked);
    }

    pub fn to_back(&mut self, ids: &[u64]) {
        let (picked, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.widgets)
            .into_iter()
            .partition(|w| ids.contains(&w.id));
        self.widgets = picked;
        self.widgets.extend(rest);
    }

    /// Ortak bir kenara hizalar. Tek bileşen pencereye göre hizalanır.
    /// Çerçeveler içindekileri de birlikte götürür.
    pub fn align(&mut self, ids: &[u64], op: AlignOp) {
        let rects: Vec<(u64, Rect)> = ids
            .iter()
            .filter_map(|id| self.get(*id).map(|w| (*id, w.rect())))
            .collect();
        let bounds = match rects.as_slice() {
            [] => return,
            [_] => self.client_rect(),
            many => many.iter().map(|(_, r)| *r).reduce(Rect::union).unwrap_or(Rect::NOTHING),
        };
        for (id, r) in rects {
            let delta = match op {
                AlignOp::Left => vec2(bounds.left() - r.left(), 0.0),
                AlignOp::CenterX => vec2(bounds.center().x - r.center().x, 0.0),
                AlignOp::Right => vec2(bounds.right() - r.right(), 0.0),
                AlignOp::Top => vec2(0.0, bounds.top() - r.top()),
                AlignOp::CenterY => vec2(0.0, bounds.center().y - r.center().y),
                AlignOp::Bottom => vec2(0.0, bounds.bottom() - r.bottom()),
            };
            let mut moving = vec![id];
            moving.extend(self.contents(id).into_iter().filter(|c| !ids.contains(c)));
            for m in moving {
                if let Some(w) = self.get_mut(m) {
                    let rect = w.rect();
                    w.set_rect(rect.translate(delta));
                }
            }
        }
    }

    /// Aradaki boşlukları eşitler (en az üç bileşen).
    pub fn distribute(&mut self, ids: &[u64], horizontal: bool) {
        let mut items: Vec<(u64, Rect)> = ids
            .iter()
            .filter_map(|id| self.get(*id).map(|w| (*id, w.rect())))
            .collect();
        if items.len() < 3 {
            return;
        }
        let start = |r: &Rect| if horizontal { r.left() } else { r.top() };
        let size = |r: &Rect| if horizontal { r.width() } else { r.height() };
        items.sort_by(|a, b| start(&a.1).total_cmp(&start(&b.1)));
        let first = items[0].1;
        let last = items[items.len() - 1].1;
        let span = start(&last) + size(&last) - start(&first);
        let total: f32 = items.iter().map(|(_, r)| size(r)).sum();
        let gap = (span - total) / (items.len() - 1) as f32;
        let mut cursor = start(&first);
        for (id, r) in items {
            let delta = cursor - start(&r);
            cursor += size(&r) + gap;
            let delta = if horizontal { vec2(delta, 0.0) } else { vec2(0.0, delta) };
            if let Some(w) = self.get_mut(id) {
                w.set_rect(r.translate(delta));
            }
        }
    }

    /// İlk seçilenin enini ya da boyunu diğerlerine verir.
    pub fn match_size(&mut self, ids: &[u64], width: bool) {
        let Some((w0, h0)) = ids.first().and_then(|id| self.get(*id)).map(|w| (w.w, w.h)) else {
            return;
        };
        for id in &ids[1..] {
            if let Some(w) = self.get_mut(*id) {
                if width {
                    w.w = w0;
                } else {
                    w.h = h0;
                }
            }
        }
    }

    /// Dosyadan okunan belgeyi tutarlı hâle getirir.
    pub fn normalize(&mut self) {
        let max_id = self.widgets.iter().map(|w| w.id).max().unwrap_or(0);
        self.next_id = self.next_id.max(max_id + 1);
        let mut seen = HashSet::new();
        for w in &mut self.widgets {
            if !seen.insert(w.id) {
                w.id = self.next_id;
                self.next_id += 1;
            }
            let rect = w.rect();
            w.set_rect(rect);
        }
        self.width = self.width.round().max(MIN_WINDOW.x);
        self.height = self.height.round().max(MIN_WINDOW.y);
        self.fix_radio_groups();
    }
}

/// `dugme_3` → `dugme`. Sonunda sayı yoksa adın kendisi.
fn name_stem(name: &str) -> &str {
    match name.rsplit_once('_') {
        Some((stem, n)) if !stem.is_empty() && !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) => {
            stem
        }
        _ => name,
    }
}

/// Rust'ta tanımlayıcı olarak kullanılamayan adlar.
const RESERVED: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "default", "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if",
    "impl", "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub",
    "ref", "return", "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

/// Yazarken kullanılan yumuşak hâl: harfler sadeleşir, geçersiz karakterler `_` olur.
pub fn soft_ident(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'ç' | 'Ç' => 'c',
            'ğ' | 'Ğ' => 'g',
            'ı' | 'İ' => 'i',
            'ö' | 'Ö' => 'o',
            'ş' | 'Ş' => 's',
            'ü' | 'Ü' => 'u',
            c if c.is_ascii_alphanumeric() => c.to_ascii_lowercase(),
            _ => '_',
        })
        .collect()
}

/// Geçerli bir Rust tanımlayıcısı: `Kaydet düğmesi` → `kaydet_dugmesi`.
pub fn ident(s: &str) -> String {
    let mut out = String::new();
    for c in soft_ident(s).chars() {
        if c != '_' {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('_') {
            out.push('_');
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("field");
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    if RESERVED.contains(&out.as_str()) {
        out.push('_');
    }
    out
}

/// Pencere başlığından tür adı: `Görsel dönüştürücü` → `GorselDonusturucu`.
pub fn type_name(title: &str, lang: Lang) -> String {
    if !title.chars().any(char::is_alphanumeric) {
        return lang.pick("App", "Uygulama").to_owned();
    }
    let name: String = ident(title)
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        })
        .collect();
    let taken = ["Self", "String", "Option", "Vec", "Box", "Default", "Result", "Some", "None", "Ok", "Err"];
    if name.is_empty() || name.starts_with(|c: char| c.is_ascii_digit()) || taken.contains(&name.as_str()) {
        format!("{}{name}", lang.pick("Window", "Pencere"))
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), vec2(10.0, 10.0))
    }

    #[test]
    fn idents_are_valid_rust() {
        assert_eq!(ident("Kaydet düğmesi"), "kaydet_dugmesi");
        assert_eq!(ident("  Çıkış  "), "cikis");
        assert_eq!(ident("2. adım"), "_2_adim");
        assert_eq!(ident("type"), "type_");
        assert_eq!(ident("***"), "field");
        assert_eq!(ident(&ident("Şu an")), ident("Şu an"));
    }

    #[test]
    fn type_names_are_pascal_case() {
        assert_eq!(type_name("Image converter", Lang::En), "ImageConverter");
        assert_eq!(type_name("Görsel dönüştürücü", Lang::Tr), "GorselDonusturucu");
        assert_eq!(type_name("", Lang::En), "App");
        assert_eq!(type_name("", Lang::Tr), "Uygulama");
        assert_eq!(type_name("3D viewer", Lang::En), "Window3dViewer");
        assert_eq!(type_name("string", Lang::Tr), "PencereString");
    }

    #[test]
    fn new_widgets_are_named_in_the_ui_language() {
        let mut doc = Document::new(Lang::En);
        let a = doc.add(Kind::Button, square(), Lang::En);
        let b = doc.add(Kind::Button, square(), Lang::Tr);
        assert_eq!((doc.get(a).unwrap().name.as_str(), doc.get(a).unwrap().text.as_str()), ("button_1", "Button"));
        assert_eq!((doc.get(b).unwrap().name.as_str(), doc.get(b).unwrap().text.as_str()), ("dugme_1", "Düğme"));
        assert_eq!(Document::new(Lang::Tr).title, "Yeni pencere");
    }

    #[test]
    fn tool_keys_are_unique_in_each_language() {
        for lang in [Lang::En, Lang::Tr] {
            let mut keys: Vec<_> = Kind::ALL.iter().map(|k| k.key(lang)).collect();
            keys.sort_by_key(|k| k.name());
            keys.dedup();
            assert_eq!(keys.len(), Kind::ALL.len(), "{lang:?} dilinde aynı harfe iki araç bağlı");
        }
    }

    #[test]
    fn z_order_moves_one_step() {
        let mut doc = Document::default();
        let a = doc.add(Kind::Label, square(), Lang::En);
        let b = doc.add(Kind::Button, square(), Lang::En);
        let c = doc.add(Kind::Entry, square(), Lang::En);
        doc.raise(&[a]);
        assert_eq!(doc.widgets.iter().map(|w| w.id).collect::<Vec<_>>(), [b, a, c]);
        doc.to_back(&[c]);
        assert_eq!(doc.widgets.iter().map(|w| w.id).collect::<Vec<_>>(), [c, b, a]);
        doc.lower(&[a]);
        assert_eq!(doc.widgets.iter().map(|w| w.id).collect::<Vec<_>>(), [c, a, b]);
    }

    #[test]
    fn radio_groups_keep_one_selection() {
        let mut doc = Document::default();
        let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 20.0));
        let a = doc.add(Kind::Radio, r, Lang::En);
        let b = doc.add(Kind::Radio, r, Lang::En);
        assert!(doc.get(a).unwrap().checked);
        assert!(!doc.get(b).unwrap().checked);
        doc.select_radio(b);
        assert_eq!(doc.radio_initial("choice"), 1);
        let copies = doc.duplicate(&[b], Vec2::ZERO);
        assert!(!doc.get(copies[0]).unwrap().checked);
        doc.remove(&[b]);
        assert!(doc.get(a).unwrap().checked);
    }

    #[test]
    fn copies_get_fresh_names() {
        for (lang, expected) in [(Lang::En, "browse_1"), (Lang::Tr, "gozat_1")] {
            let mut doc = Document::example(lang);
            let id = doc.widgets[2].id;
            let copy = doc.duplicate(&[id], vec2(16.0, 16.0))[0];
            assert_eq!(doc.get(copy).unwrap().name, expected);
            assert_eq!(doc.get(copy).unwrap().x, doc.get(id).unwrap().x + 16.0);
        }
    }
}
