//! Belgeden, `cargo run` ile çalışan tek dosyalık bir eframe uygulaması üretir.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

use crate::i18n::Lang;
use crate::model::{Document, HAlign, Kind, Widget, ident, type_name};

/// Üretilen kodun hedeflediği eframe sürümü.
pub const EFRAME: &str = "0.37";

const LIVE: &str = include_str!("live.rs");

/// Yardımcıların çıktıdaki sırası.
const HELPER_ORDER: [&str; 12] = [
    "at", "label", "button", "text_field", "text_area", "checkbox", "radio", "combo", "list",
    "slider", "progress", "group",
];

/// `live.rs` yorumlarının Türkçe çıktıdaki karşılıkları.
pub const HELPER_DOCS_TR: [(&str, &str); 5] = [
    (
        "/// Position inside the window: (x, y) from corner `o`, size (w, h).",
        "/// Pencere içi konum: `o` köşesine göre (x, y), boyut (en, boy).",
    ),
    (
        "/// Centers the text vertically and aligns it horizontally to `align`.",
        "/// Metni alanın içinde dikeyde ortalar, yatayda `align`'a yaslar.",
    ),
    (
        "/// Multi-line text that scrolls inside its area when it overflows.",
        "/// Çok satırlı metin; içerik taşarsa alan kendi içinde kayar.",
    ),
    (
        "/// Option number `index` of a group; clicking it sets `value` to `index`.",
        "/// Gruptaki `index`'inci seçenek; tıklanınca `value` ona eşitlenir.",
    ),
    (
        "/// A thin outlined frame; the title sits on its top border.",
        "/// İnce çizgili çerçeve; başlık üst çizginin üstüne oturur.",
    ),
];

/// `live.rs` içindeki `// @ad` bölümleri: `pub` önekleri atılmış, yorumları `lang` dilinde.
fn helper_sources(lang: Lang) -> HashMap<&'static str, String> {
    let mut out = HashMap::new();
    let mut current: Option<(&'static str, String)> = None;
    for line in LIVE.lines() {
        if let Some(name) = line.strip_prefix("// @") {
            if let Some((done, body)) = current.take() {
                out.insert(done, body.trim_end().to_owned());
            }
            current = Some((name.trim(), String::new()));
        } else if let Some((_, body)) = current.as_mut() {
            let doc = HELPER_DOCS_TR.iter().find(|(en, _)| *en == line);
            match (line.strip_prefix("pub fn "), doc) {
                (Some(rest), _) => body.push_str(&format!("fn {rest}")),
                (None, Some((_, tr))) if lang == Lang::Tr => body.push_str(tr),
                _ => body.push_str(line),
            }
            body.push('\n');
        }
    }
    if let Some((done, body)) = current {
        out.insert(done, body.trim_end().to_owned());
    }
    out
}

struct Field {
    name: String,
    ty: &'static str,
    init: String,
}

/// Hangi bileşen hangi alana ve yönteme bağlanıyor.
#[derive(Default)]
struct Plan {
    fields: Vec<Field>,
    field_of: HashMap<u64, String>,
    group_field: HashMap<String, String>,
    radio_index: HashMap<u64, usize>,
    /// Düğme yöntemleri: (ad, düğme metni).
    methods: Vec<(String, String)>,
    method_of: HashMap<u64, String>,
}

/// Ad kullanılmışsa sonuna `_2`, `_3`... ekler.
fn unique(base: String, used: &mut HashSet<String>) -> String {
    let mut name = base.clone();
    let mut n = 2;
    while !used.insert(name.clone()) {
        name = format!("{base}_{n}");
        n += 1;
    }
    name
}

impl Plan {
    fn new(doc: &Document) -> Self {
        let mut plan = Plan::default();
        let mut fields = HashSet::new();
        let mut methods = HashSet::new();
        let mut radios_seen: HashMap<&str, usize> = HashMap::new();
        for w in &doc.widgets {
            let field = match w.kind {
                Kind::Entry | Kind::TextArea => Some(("String", string_init(&w.text))),
                Kind::Checkbox => Some(("bool", w.checked.to_string())),
                Kind::Combo => {
                    let last = w.clean_items().len().saturating_sub(1);
                    Some(("usize", w.selected.min(last).to_string()))
                }
                Kind::List => Some(("Option<usize>", "None".to_owned())),
                Kind::Slider => Some(("f32", float(w.value.clamp(w.min.min(w.max), w.max.max(w.min))))),
                Kind::Progress => Some(("f32", float((w.value / 100.0).clamp(0.0, 1.0)))),
                Kind::Radio => {
                    let index = radios_seen.entry(&w.group).or_default();
                    plan.radio_index.insert(w.id, *index);
                    *index += 1;
                    if !plan.group_field.contains_key(&w.group) {
                        let name = unique(ident(&w.group), &mut fields);
                        plan.group_field.insert(w.group.clone(), name.clone());
                        plan.fields.push(Field {
                            name,
                            ty: "usize",
                            init: doc.radio_initial(&w.group).to_string(),
                        });
                    }
                    None
                }
                Kind::Button => {
                    let name = unique(ident(&w.name), &mut methods);
                    plan.method_of.insert(w.id, name.clone());
                    plan.methods.push((name, w.text.replace('\n', " ")));
                    None
                }
                Kind::Label | Kind::Group => None,
            };
            if let Some((ty, init)) = field {
                let name = unique(ident(&w.name), &mut fields);
                plan.field_of.insert(w.id, name.clone());
                plan.fields.push(Field { name, ty, init });
            }
        }
        plan
    }
}

/// Rust dizgi değişmezi.
fn lit(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => {}
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn string_init(s: &str) -> String {
    if s.is_empty() {
        "String::new()".to_owned()
    } else {
        format!("{}.to_owned()", lit(s))
    }
}

/// `24.0`, `0.4` gibi; tam sayılar `.0` ile biter.
fn float(v: f32) -> String {
    if v.fract() == 0.0 && v.abs() < 1e7 {
        format!("{v:.1}")
    } else {
        format!("{v}")
    }
}

fn items(w: &Widget) -> String {
    let parts: Vec<String> = w.clean_items().into_iter().map(lit).collect();
    format!("&[{}]", parts.join(", "))
}

/// `ad(arg, ...)` çağrısını yazar; satır 100 karakteri aşarsa rustfmt gibi böler.
fn call(out: &mut String, indent: usize, name: &str, args: &[String], tail: &str) {
    let pad = "    ".repeat(indent);
    let one_line = format!("{pad}{name}({}){tail}", args.join(", "));
    if one_line.chars().count() <= 100 {
        out.push_str(&one_line);
        out.push('\n');
        return;
    }
    let _ = writeln!(out, "{pad}{name}(");
    for arg in args {
        let _ = writeln!(out, "{pad}    {arg},");
    }
    let _ = writeln!(out, "{pad}){tail}");
}

/// Tasarımı tek dosyalık bir eframe uygulamasına çevirir. Yorumlar `lang` dilindedir.
pub fn generate(doc: &Document, lang: Lang) -> String {
    let plan = Plan::new(doc);
    let ty = type_name(&doc.title, lang);
    let mut used: HashSet<&str> = HashSet::new();

    let mut body = String::new();
    for w in &doc.widgets {
        let r = format!("at(o, {}, {}, {}, {})", float(w.x), float(w.y), float(w.w), float(w.h));
        let field = || plan.field_of[&w.id].clone();
        let (helper, args): (&str, Vec<String>) = match w.kind {
            Kind::Label => {
                let align = match w.align {
                    HAlign::Left => "egui::Align::LEFT",
                    HAlign::Center => "egui::Align::Center",
                    HAlign::Right => "egui::Align::RIGHT",
                };
                ("label", vec![r, lit(&w.text), align.to_owned()])
            }
            Kind::Button => {
                used.insert("button");
                let _ = writeln!(body, "        if button(ui, {r}, {}).clicked() {{", lit(&w.text));
                let _ = writeln!(body, "            self.{}();", plan.method_of[&w.id]);
                let _ = writeln!(body, "        }}");
                continue;
            }
            Kind::Entry => ("text_field", vec![r, format!("&mut self.{}", field())]),
            Kind::TextArea => ("text_area", vec![r, lit(&field()), format!("&mut self.{}", field())]),
            Kind::Checkbox => ("checkbox", vec![r, format!("&mut self.{}", field()), lit(&w.text)]),
            Kind::Radio => (
                "radio",
                vec![
                    r,
                    format!("&mut self.{}", plan.group_field[&w.group]),
                    plan.radio_index[&w.id].to_string(),
                    lit(&w.text),
                ],
            ),
            Kind::Combo => ("combo", vec![r, lit(&field()), format!("&mut self.{}", field()), items(w)]),
            Kind::List => ("list", vec![r, lit(&field()), format!("&mut self.{}", field()), items(w)]),
            Kind::Slider => {
                let (lo, hi) = (w.min.min(w.max), w.max.max(w.min));
                let range = format!("{}..={}", float(lo), float(hi));
                ("slider", vec![r, format!("&mut self.{}", field()), range])
            }
            Kind::Progress => ("progress", vec![r, format!("self.{}", field())]),
            Kind::Group => ("group", vec![r, lit(&w.text)]),
        };
        used.insert(helper);
        let mut all = vec!["ui".to_owned()];
        all.extend(args);
        call(&mut body, 2, helper, &all, ";");
    }
    if !doc.widgets.is_empty() {
        used.insert("at");
    }

    let mut s = String::new();
    let made_with = lang.pick("designed with Pafta.", "Pafta ile tasarlandı.");
    let _ = writeln!(s, "// {}: {made_with}", doc.title.replace('\n', " "));
    let _ = writeln!(s, "// Cargo.toml → [dependencies] eframe = \"{EFRAME}\"");
    let _ = writeln!(s);
    let _ = writeln!(s, "use eframe::egui;");
    let _ = writeln!(s);
    let _ = writeln!(s, "fn main() -> eframe::Result {{");
    let _ = writeln!(s, "    let options = eframe::NativeOptions {{");
    let _ = writeln!(s, "        viewport: egui::ViewportBuilder::default()");
    let _ = writeln!(s, "            .with_title({})", lit(&doc.title));
    let _ = writeln!(s, "            .with_inner_size([{}, {}])", float(doc.width), float(doc.height));
    let _ = writeln!(s, "            .with_resizable(false),");
    let _ = writeln!(s, "        ..Default::default()");
    let _ = writeln!(s, "    }};");
    let _ = writeln!(s, "    eframe::run_native(");
    let _ = writeln!(s, "        {},", lit(&doc.title));
    let _ = writeln!(s, "        options,");
    let _ = writeln!(s, "        Box::new(|cc| {{");
    let _ = writeln!(s, "            cc.egui_ctx.set_theme(egui::Theme::Light);");
    let _ = writeln!(s, "            Ok(Box::new({ty}::default()))");
    let _ = writeln!(s, "        }}),");
    let _ = writeln!(s, "    )");
    let _ = writeln!(s, "}}");
    let _ = writeln!(s);

    if plan.fields.is_empty() {
        let _ = writeln!(s, "#[derive(Default)]");
        let _ = writeln!(s, "struct {ty};");
    } else {
        let _ = writeln!(s, "struct {ty} {{");
        for f in &plan.fields {
            let _ = writeln!(s, "    {}: {},", f.name, f.ty);
        }
        let _ = writeln!(s, "}}");
        let _ = writeln!(s);
        let _ = writeln!(s, "impl Default for {ty} {{");
        let _ = writeln!(s, "    fn default() -> Self {{");
        let _ = writeln!(s, "        Self {{");
        for f in &plan.fields {
            let _ = writeln!(s, "            {}: {},", f.name, f.init);
        }
        let _ = writeln!(s, "        }}");
        let _ = writeln!(s, "    }}");
        let _ = writeln!(s, "}}");
    }

    if !plan.methods.is_empty() {
        let _ = writeln!(s);
        let _ = writeln!(s, "impl {ty} {{");
        for (i, (name, text)) in plan.methods.iter().enumerate() {
            if i > 0 {
                let _ = writeln!(s);
            }
            let about = lang.pick(
                format!("Called when “{text}” is clicked."),
                format!("“{text}” düğmesine basıldı."),
            );
            let _ = writeln!(s, "    /// {about}");
            let _ = writeln!(s, "    fn {name}(&mut self) {{}}");
        }
        let _ = writeln!(s, "}}");
    }

    let _ = writeln!(s);
    let _ = writeln!(s, "impl eframe::App for {ty} {{");
    let _ = writeln!(s, "    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {{");
    let _ = writeln!(s, "        ui.painter().rect_filled(ui.max_rect(), 0.0, ui.visuals().panel_fill);");
    if !body.is_empty() {
        let _ = writeln!(s, "        let o = ui.max_rect().min;");
        let _ = writeln!(s);
        s.push_str(&body);
    }
    let _ = writeln!(s, "    }}");
    let _ = writeln!(s, "}}");

    let sources = helper_sources(lang);
    for name in HELPER_ORDER {
        if used.contains(name) {
            let _ = writeln!(s);
            s.push_str(&sources[name]);
            s.push('\n');
        }
    }
    s
}

/// Dışa aktarılan Cargo projesinin bildirimi.
pub fn cargo_toml(doc: &Document) -> String {
    format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\neframe = \"{EFRAME}\"\n",
        ident(&doc.title)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_helper_has_a_source() {
        let sources = helper_sources(Lang::En);
        for name in HELPER_ORDER {
            let src = &sources[name];
            assert!(src.contains(&format!("fn {name}(")), "{name} bulunamadı");
            assert!(!src.contains("pub fn"), "{name} içinde pub kaldı");
        }
    }

    #[test]
    fn every_helper_comment_has_a_turkish_line() {
        for line in LIVE.lines().filter(|l| l.starts_with("///")) {
            assert!(
                HELPER_DOCS_TR.iter().any(|(en, _)| *en == line),
                "Türkçe karşılığı yok: {line}"
            );
        }
        let tr = helper_sources(Lang::Tr);
        for name in HELPER_ORDER {
            assert!(
                !HELPER_DOCS_TR.iter().any(|(en, _)| tr[name].contains(en)),
                "{name} Türkçe çıktıda İngilizce kaldı"
            );
        }
    }

    #[test]
    fn example_design_becomes_an_app() {
        let code = generate(&Document::example(Lang::En), Lang::En);
        assert!(code.contains("// Image converter: designed with Pafta."));
        assert!(code.contains("struct ImageConverter {"));
        assert!(code.contains(r#"source: "D:\\Photos\\2026".to_owned(),"#));
        assert!(code.contains("output: 0,"));
        assert!(code.contains("progress: 0.4,"));
        assert!(code.contains(r#"if button(ui, at(o, 320.0, 308.0, 96.0, 28.0), "Convert").clicked() {"#));
        assert!(code.contains("    /// Called when “Convert” is clicked.\n    fn convert(&mut self) {}"));
        assert!(code.contains("self.convert();"));
        assert!(code.contains(r#"radio(ui, at(o, 232.0, 216.0, 168.0, 22.0), &mut self.output, 1, "New folder");"#));
        assert!(code.contains("/// A thin outlined frame; the title sits on its top border."));
        assert!(!code.contains("fn text_area("), "kullanılmayan yardımcı eklenmez");
    }

    #[test]
    fn turkish_output_has_turkish_comments() {
        let code = generate(&Document::example(Lang::Tr), Lang::Tr);
        assert!(code.contains("// Görsel dönüştürücü: Pafta ile tasarlandı."));
        assert!(code.contains("struct GorselDonusturucu {"));
        assert!(code.contains("    /// “Dönüştür” düğmesine basıldı.\n    fn donustur(&mut self) {}"));
        assert!(code.contains("/// İnce çizgili çerçeve; başlık üst çizginin üstüne oturur."));
        assert!(!code.contains("designed with Pafta"));
    }

    #[test]
    fn empty_design_is_a_unit_struct() {
        let code = generate(&Document::default(), Lang::En);
        assert!(code.contains("#[derive(Default)]\nstruct NewWindow;"));
        assert!(!code.contains("let o ="));
    }

    #[test]
    fn clashing_names_are_made_unique() {
        use eframe::egui::{Rect, pos2, vec2};
        let mut doc = Document::default();
        let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(10.0, 10.0));
        let a = doc.add(Kind::Entry, r, Lang::En);
        let b = doc.add(Kind::Entry, r, Lang::En);
        doc.get_mut(a).unwrap().name = "name".into();
        doc.get_mut(b).unwrap().name = "Name".into();
        let code = generate(&doc, Lang::En);
        assert!(code.contains("    name: String,\n    name_2: String,"));
    }
}
