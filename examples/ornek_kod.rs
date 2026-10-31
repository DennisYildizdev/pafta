//! Bir tasarımın ürettiği kodu yazdırır:
//! `cargo run --example ornek_kod > main.rs` örnek tasarımı,
//! `cargo run --example ornek_kod -- all > main.rs` her bileşen türünden birini yazar.
//! `tr` eklenirse adlar ve yorumlar Türkçe olur.

use eframe::egui::{Rect, pos2, vec2};
use pafta::i18n::Lang;
use pafta::model::{Document, HAlign, Kind};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let lang = if args.iter().any(|a| a == "tr") { Lang::Tr } else { Lang::En };
    let doc = if args.iter().any(|a| a == "all") {
        every_kind(lang)
    } else {
        Document::example(lang)
    };
    print!("{}", pafta::codegen::generate(&doc, lang));
}

/// Her türden bir bileşen, alt alta.
fn every_kind(lang: Lang) -> Document {
    let mut doc = Document {
        title: lang.pick("All components", "Bütün bileşenler").to_owned(),
        width: 360.0,
        ..Document::new(lang)
    };
    let mut y = 16.0;
    for kind in Kind::ALL {
        let size = kind.default_size();
        doc.add(kind, Rect::from_min_size(pos2(16.0, y), size), lang);
        if kind == Kind::Label {
            for (align, dx) in [(HAlign::Center, 120.0), (HAlign::Right, 240.0)] {
                let other = doc.add(kind, Rect::from_min_size(pos2(16.0 + dx, y), vec2(100.0, 20.0)), lang);
                doc.get_mut(other).unwrap().align = align;
            }
        }
        y += size.y + 12.0;
    }
    doc.height = y + 16.0;
    doc
}
