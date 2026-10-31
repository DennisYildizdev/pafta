//! Üretilen Rust kodu için küçük bir renklendirici (Xcode renkleri).

use eframe::egui::{
    Color32, FontId,
    text::{LayoutJob, TextFormat},
};

pub struct Palette {
    pub plain: Color32,
    pub keyword: Color32,
    pub string: Color32,
    pub comment: Color32,
    pub number: Color32,
    pub ty: Color32,
}

impl Palette {
    pub fn new(dark: bool, plain: Color32) -> Self {
        let c = |light: [u8; 3], dark_rgb: [u8; 3]| {
            let [r, g, b] = if dark { dark_rgb } else { light };
            Color32::from_rgb(r, g, b)
        };
        Self {
            plain,
            keyword: c([0x9b, 0x23, 0x93], [0xfc, 0x5f, 0xa3]),
            string: c([0xc4, 0x1a, 0x16], [0xfc, 0x6a, 0x5d]),
            comment: c([0x5d, 0x6c, 0x79], [0x6c, 0x79, 0x86]),
            number: c([0x1c, 0x00, 0xcf], [0xd0, 0xbf, 0x69]),
            ty: c([0x0b, 0x4f, 0x79], [0x5d, 0xd8, 0xff]),
        }
    }
}

const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "false", "fn", "for", "if", "impl",
    "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self",
    "static", "struct", "super", "trait", "true", "type", "use", "where", "while", "dyn",
];

pub fn highlight(code: &str, font: FontId, palette: &Palette) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mut rest = code;
    while !rest.is_empty() {
        let (len, color) = token(rest, palette);
        job.append(&rest[..len], 0.0, TextFormat::simple(font.clone(), color));
        rest = &rest[len..];
    }
    job
}

/// Baştaki parçanın bayt uzunluğu ve rengi.
fn token(s: &str, p: &Palette) -> (usize, Color32) {
    if s.starts_with("//") {
        return (s.find('\n').unwrap_or(s.len()), p.comment);
    }
    let first = s.chars().next().unwrap_or(' ');
    if first == '"' {
        let mut escaped = false;
        for (i, c) in s.char_indices().skip(1) {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => return (i + 1, p.string),
                _ => {}
            }
        }
        return (s.len(), p.string);
    }
    if first.is_ascii_digit() {
        let bytes = s.as_bytes();
        let mut end = 0;
        while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b'_') {
            end += 1;
        }
        if end + 1 < bytes.len() && bytes[end] == b'.' && bytes[end + 1].is_ascii_digit() {
            end += 1;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
        }
        return (end, p.number);
    }
    if first.is_alphabetic() || first == '_' {
        let end = s
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(s.len());
        let word = &s[..end];
        let color = if KEYWORDS.contains(&word) {
            p.keyword
        } else if word.starts_with(|c: char| c.is_ascii_uppercase()) {
            p.ty
        } else {
            p.plain
        };
        return (end, color);
    }
    // Noktalama ve boşluklar: bir sonraki anlamlı parçaya kadar.
    let end = s
        .char_indices()
        .skip(1)
        .find(|(_, c)| c.is_alphanumeric() || *c == '_' || *c == '"' || *c == '/')
        .map_or(s.len(), |(i, _)| i);
    (end, p.plain)
}
