//! Renk belirteçleri, egui stili, yazı tipleri ve uygulama simgesi.

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, FontTweak, Margin,
    Shadow, Stroke, Style, TextStyle, Theme, Visuals, epaint::text::VariationCoords,
    style::WidgetVisuals, vec2,
};

use crate::model::Family;

/// Arayüzün bütün renkleri. Açık ve koyu tema aynı adları farklı değerlerle doldurur.
#[derive(Clone, Copy, Debug)]
pub struct Tokens {
    pub dark: bool,
    /// Pencerenin çevresindeki tuval zemini.
    pub canvas: Color32,
    /// Kenar çubukları ve üst çubuk.
    pub surface: Color32,
    pub hover: Color32,
    pub separator: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub text: Color32,
    pub text2: Color32,
    pub text3: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    /// Seçili satırların zemini.
    pub accent_tint: Color32,
    pub on_accent: Color32,
    /// Hizalama ve ölçü çizgileri.
    pub guide: Color32,
    pub danger: Color32,
    pub field: Color32,
    pub control: Color32,
    pub control_hover: Color32,
    pub control_active: Color32,
    /// Bölümlü düğmede seçili bölüm.
    pub raised: Color32,
}

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

pub const LIGHT: Tokens = Tokens {
    dark: false,
    canvas: hex(0xf5f5f7),
    surface: hex(0xffffff),
    hover: hex(0xf2f2f4),
    separator: hex(0xe5e5ea),
    border: hex(0xd2d2d7),
    border_strong: hex(0xaeaeb2),
    text: hex(0x1d1d1f),
    text2: hex(0x6e6e73),
    text3: hex(0x8e8e93),
    accent: hex(0x0071e3),
    accent_hover: hex(0x0077ed),
    accent_tint: hex(0xe0eefc),
    on_accent: hex(0xffffff),
    guide: hex(0xff2d55),
    danger: hex(0xd70015),
    field: hex(0xffffff),
    control: hex(0xf0f0f2),
    control_hover: hex(0xe8e8eb),
    control_active: hex(0xdcdce0),
    raised: hex(0xffffff),
};

pub const DARK: Tokens = Tokens {
    dark: true,
    canvas: hex(0x111113),
    surface: hex(0x1c1c1e),
    hover: hex(0x2c2c2e),
    separator: hex(0x38383a),
    border: hex(0x48484a),
    border_strong: hex(0x636366),
    text: hex(0xf5f5f7),
    text2: hex(0x98989d),
    text3: hex(0x6e6e73),
    accent: hex(0x0a84ff),
    accent_hover: hex(0x409cff),
    accent_tint: hex(0x183554),
    on_accent: hex(0xffffff),
    guide: hex(0xff375f),
    danger: hex(0xff453a),
    field: hex(0x262628),
    control: hex(0x3a3a3c),
    control_hover: hex(0x444446),
    control_active: hex(0x505053),
    raised: hex(0x636366),
};

pub fn tokens(ctx: &egui::Context) -> Tokens {
    match ctx.theme() {
        Theme::Dark => DARK,
        Theme::Light => LIGHT,
    }
}

impl Tokens {
    /// Bileşen ailesinin kimlik rengi (Apple sistem renkleri). Yalnız küçük işaretlerde kullanılır.
    pub fn family(&self, family: Family) -> Color32 {
        let (light, dark) = match family {
            Family::Text => (0x5856d6, 0x5e5ce6),
            Family::Action => (0xff9500, 0xff9f0a),
            Family::Input => (0x34c759, 0x30d158),
            Family::Choice => (0xaf52de, 0xbf5af2),
            Family::Value => (0x30b0c7, 0x40c8e0),
            Family::Layout => (0xa2845e, 0xac8e68),
        };
        hex(if self.dark { dark } else { light })
    }
}

const SEMIBOLD: &str = "semibold";
const DISPLAY: &str = "display";

pub fn body(size: f32) -> FontId {
    FontId::proportional(size)
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

/// Büyük başlıklar için: Segoe UI Variable'ın "Display" optik boyutu.
pub fn display(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(DISPLAY.into()))
}

pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

/// Yazı tiplerini ve iki temanın stilini kurar.
pub fn install(ctx: &egui::Context) {
    install_fonts(ctx);
    ctx.set_style_of(Theme::Light, style(&LIGHT));
    ctx.set_style_of(Theme::Dark, style(&DARK));
    // Ctrl + artı/eksi tuvali yakınlaştırır, arayüzü değil.
    ctx.options_mut(|o| o.zoom_with_keyboard = false);
}

/// Windows'ta Segoe UI Variable ve Cascadia Mono'yu yükler; yoksa egui'nin yazı tiplerine düşer.
fn install_fonts(ctx: &egui::Context) {
    let dir = std::env::var_os("WINDIR")
        .map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from)
        .join("Fonts");
    let read = |file: &str| std::fs::read(dir.join(file)).ok();

    let mut defs = FontDefinitions::default();
    let mut regular: Vec<&str> = Vec::new();
    let mut strong: Vec<&str> = Vec::new();
    let mut large: Vec<&str> = Vec::new();

    if let Some(bytes) = read("SegUIVar.ttf") {
        // Tek dosya, üç kesim: ağırlık (wght) ve optik boyut (opsz) eksenleri.
        let blob = Arc::new(bytes);
        let face = |weight: f32, optical: f32| {
            let coords = VariationCoords::new([(b"wght", weight), (b"opsz", optical)]);
            let tweak = FontTweak {
                coords,
                ..Default::default()
            };
            Arc::new(FontData::from_blob(blob.clone(), 0).tweak(tweak))
        };
        defs.font_data.insert("segoe".into(), face(400.0, 10.5));
        defs.font_data.insert("segoe-semibold".into(), face(600.0, 10.5));
        defs.font_data.insert("segoe-display".into(), face(600.0, 28.0));
        regular.push("segoe");
        strong.push("segoe-semibold");
        large.push("segoe-display");
    } else {
        if let Some(bytes) = read("segoeui.ttf") {
            defs.font_data.insert("segoe".into(), Arc::new(FontData::from_owned(bytes)));
            regular.push("segoe");
        }
        if let Some(bytes) = read("seguisb.ttf") {
            defs.font_data.insert("segoe-semibold".into(), Arc::new(FontData::from_owned(bytes)));
            strong.push("segoe-semibold");
        }
    }
    if strong.is_empty() {
        strong = regular.clone();
    }
    if large.is_empty() {
        large = strong.clone();
    }

    let symbols = read("seguisym.ttf").map(|bytes| {
        defs.font_data.insert("segoe-symbol".into(), Arc::new(FontData::from_owned(bytes)));
        "segoe-symbol".to_owned()
    });
    let fallback = defs.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let chain = |first: &[&str]| -> Vec<String> {
        first
            .iter()
            .map(|name| (*name).to_owned())
            .chain(symbols.clone())
            .chain(fallback.iter().cloned())
            .collect()
    };
    defs.families.insert(FontFamily::Proportional, chain(&regular));
    defs.families.insert(FontFamily::Name(SEMIBOLD.into()), chain(&strong));
    defs.families.insert(FontFamily::Name(DISPLAY.into()), chain(&large));

    if let Some(bytes) = read("CascadiaMono.ttf").or_else(|| read("consola.ttf")) {
        defs.font_data.insert("mono".into(), Arc::new(FontData::from_owned(bytes)));
        defs.families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "mono".into());
    }
    ctx.set_fonts(defs);
}

fn style(t: &Tokens) -> Style {
    let mut style = Style {
        visuals: if t.dark { Visuals::dark() } else { Visuals::light() },
        ..Style::default()
    };
    // Yazı ölçeği: 11 / 13 / 20, kod için 12.
    style.text_styles = [
        (TextStyle::Small, body(11.0)),
        (TextStyle::Body, body(13.0)),
        (TextStyle::Button, body(13.0)),
        (TextStyle::Heading, display(20.0)),
        (TextStyle::Monospace, mono(12.0)),
    ]
    .into();
    style.interaction.selectable_labels = false;
    style.animation_time = 0.2;

    let spacing = &mut style.spacing;
    spacing.item_spacing = vec2(8.0, 8.0);
    spacing.button_padding = vec2(10.0, 4.0);
    spacing.interact_size = vec2(40.0, 26.0);
    spacing.menu_margin = Margin::same(6);
    spacing.window_margin = Margin::same(16);
    spacing.icon_width = 16.0;
    spacing.icon_width_inner = 9.0;
    spacing.icon_spacing = 6.0;

    let v = &mut style.visuals;
    v.panel_fill = t.surface;
    v.window_fill = t.surface;
    v.extreme_bg_color = t.field;
    v.text_edit_bg_color = Some(t.field);
    v.faint_bg_color = t.hover;
    v.code_bg_color = t.hover;
    v.hyperlink_color = t.accent;
    v.error_fg_color = t.danger;
    v.window_stroke = Stroke::new(1.0, t.separator);
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(10);
    v.window_shadow = Shadow {
        offset: [0, 8],
        blur: 28,
        spread: 0,
        color: Color32::from_black_alpha(if t.dark { 110 } else { 30 }),
    };
    v.popup_shadow = Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(if t.dark { 90 } else { 22 }),
    };
    v.selection.bg_fill = t.accent_tint;
    v.selection.stroke = Stroke::new(1.0, t.accent);
    v.slider_trailing_fill = true;

    let control = |fill: Color32, stroke: Color32| WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(1.0, stroke),
        corner_radius: CornerRadius::same(8),
        fg_stroke: Stroke::new(1.0, t.text),
        expansion: 0.0,
    };
    v.widgets.noninteractive = control(t.surface, t.separator);
    v.widgets.inactive = control(t.control, t.border);
    v.widgets.hovered = control(t.control_hover, t.border_strong);
    v.widgets.active = control(t.control_active, t.border_strong);
    v.widgets.open = control(t.control_hover, t.border_strong);
    style
}

/// Pencere simgesi; çizim `icon.rs` içinde, exe'ye gömülen .ico ile aynı.
pub fn icon() -> egui::IconData {
    const SIZE: u32 = 64;
    egui::IconData {
        rgba: crate::icon::rgba(SIZE),
        width: SIZE,
        height: SIZE,
    }
}
