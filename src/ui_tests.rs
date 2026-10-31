//! Tuvali sanal fareyle sınar. `PAFTA_GORUNTU=klasör` verilirse kareleri PNG olarak kaydeder.

use std::path::PathBuf;

use eframe::egui::{Event, Key, Modifiers, PointerButton, Pos2, Theme, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use crate::app::Pafta;
use crate::i18n::Lang;
use crate::model::Kind;

fn shots() -> Option<PathBuf> {
    std::env::var_os("PAFTA_GORUNTU").map(PathBuf::from)
}

fn harness(theme: Theme) -> Harness<'static, Pafta> {
    let mut builder = Harness::builder()
        .with_size(vec2(1320.0, 840.0))
        .with_theme(theme)
        .with_step_dt(1.0 / 60.0);
    if shots().is_some() {
        builder = builder.wgpu();
    }
    let mut harness = builder.build_eframe(|cc| Pafta::new(cc));
    harness.run_steps(3);
    harness
}

fn shot(h: &mut Harness<'_, Pafta>, name: &str) {
    if let Some(dir) = shots() {
        std::fs::create_dir_all(&dir).expect("klasör açılamadı");
        let image = h.render().expect("kare çizilemedi");
        image.save(dir.join(format!("{name}.png"))).expect("resim yazılamadı");
    }
}

/// Bileşenin ortasının ekrandaki yeri.
fn center_of(h: &Harness<'_, Pafta>, name: &str) -> Pos2 {
    let app = h.state();
    let w = app.doc.widgets.iter().find(|w| w.name == name).expect("bileşen yok");
    app.to_screen(w.rect().center())
}

fn rect_of(h: &Harness<'_, Pafta>, name: &str) -> [f32; 4] {
    let w = h.state().doc.widgets.iter().find(|w| w.name == name).expect("bileşen yok");
    [w.x, w.y, w.w, w.h]
}

fn click(h: &mut Harness<'_, Pafta>, pos: Pos2) {
    h.hover_at(pos);
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.step();
}

/// Basılı sürükler. `release` false ise bırakmadan durur ki ara hâl görülebilsin.
fn drag(h: &mut Harness<'_, Pafta>, from: Pos2, to: Pos2, release: bool) {
    h.hover_at(from);
    h.drag_at(from);
    for i in 1..=8 {
        h.hover_at(from + (to - from) * (i as f32 / 8.0));
    }
    h.step();
    if release {
        h.drop_at(to);
        h.step();
    }
}

#[test]
fn opens_in_english_with_the_example() {
    let h = harness(Theme::Dark);
    assert_eq!(h.state().prefs.lang, Lang::En);
    assert_eq!(h.state().doc.title, "Image converter");
    h.get_by_label("Checkbox");
    h.get_by_label("Canvas");
}

#[test]
fn dragging_snaps_to_neighbours() {
    let mut h = harness(Theme::Dark);
    shot(&mut h, "1-acilis");
    let from = center_of(&h, "convert");
    // Orta çizgi 223'e gelir; source_label'ın sağ kenarı 224'te olduğu için oraya oturur.
    drag(&mut h, from, from + vec2(-145.0, 0.0), false);
    assert!(!h.state().view.guides.is_empty(), "hizalama çizgisi görünmeli");
    shot(&mut h, "2-surukle");
    h.drop_at(from + vec2(-145.0, 0.0));
    h.step();
    assert_eq!(rect_of(&h, "convert"), [176.0, 308.0, 96.0, 28.0]);
    assert!(h.state().view.guides.is_empty(), "bırakınca çizgiler kalkar");
}

#[test]
fn alt_moves_freely() {
    let mut h = harness(Theme::Dark);
    let from = center_of(&h, "convert");
    h.event(Event::ModifiersChanged(Modifiers::ALT));
    drag(&mut h, from, from + vec2(-145.0, -3.0), true);
    assert_eq!(rect_of(&h, "convert"), [175.0, 305.0, 96.0, 28.0]);
}

#[test]
fn tool_click_creates_widget() {
    let mut h = harness(Theme::Dark);
    h.key_press(Key::B);
    h.step();
    assert_eq!(h.state().tool, Some(Kind::Button));
    let at = h.state().to_screen(pos2(33.0, 303.0));
    click(&mut h, at);
    let app = h.state();
    let w = app.doc.widgets.last().expect("yeni bileşen");
    assert_eq!((w.kind, w.name.as_str(), w.text.as_str()), (Kind::Button, "button_1", "Button"));
    assert_eq!([w.x, w.y, w.w, w.h], [32.0, 304.0, 96.0, 28.0], "ızgaraya oturur");
    assert_eq!(app.sel, [w.id]);
    assert_eq!(app.tool, None, "araç bir kullanımdan sonra bırakılır");
    shot(&mut h, "3-yeni-dugme");
}

#[test]
fn turkish_changes_labels_keys_and_names() {
    let mut h = harness(Theme::Dark);
    h.state_mut().prefs.lang = Lang::Tr;
    h.run_steps(2);
    h.get_by_label("Onay kutusu");
    h.get_by_label("Tuval");
    // Türkçede düğmenin harfi D (Düğme); İngilizcedeki B artık bir şey yapmaz.
    h.key_press(Key::B);
    h.step();
    assert_eq!(h.state().tool, None);
    h.key_press(Key::D);
    h.step();
    assert_eq!(h.state().tool, Some(Kind::Button));
    let at = h.state().to_screen(pos2(33.0, 303.0));
    click(&mut h, at);
    let w = h.state().doc.widgets.last().expect("yeni bileşen");
    assert_eq!((w.name.as_str(), w.text.as_str()), ("dugme_1", "Düğme"));
    assert!(h.state().code.text.contains("düğmesine basıldı"), "kod yorumları da Türkçe");
    shot(&mut h, "4-turkce");
}

#[test]
fn palette_drag_drops_on_canvas() {
    let mut h = harness(Theme::Dark);
    let row = h.get_by_label("Checkbox").rect().center();
    let target = h.state().to_screen(pos2(100.0, 320.0));
    h.hover_at(row);
    h.drag_at(row);
    for i in 1..=8 {
        h.hover_at(row + (target - row) * (i as f32 / 8.0));
    }
    h.step();
    assert_eq!(h.state().palette_drag, Some(Kind::Checkbox));
    shot(&mut h, "5-paletten-surukle");
    h.drop_at(target);
    h.step();
    let w = h.state().doc.widgets.last().expect("yeni bileşen");
    assert_eq!(w.kind, Kind::Checkbox);
    assert_eq!([w.x, w.y], [24.0, 312.0], "imlecin ortasına, ızgaraya");
    assert_eq!(h.state().palette_drag, None);
}

#[test]
fn double_click_edits_text_in_place() {
    let mut h = harness(Theme::Light);
    let at = center_of(&h, "source_label");
    click(&mut h, at);
    click(&mut h, at);
    assert!(h.state().view.edit.is_some(), "çift tık düzenlemeyi açar");
    h.step();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.event(Event::Text("Target folder".to_owned()));
    h.step();
    shot(&mut h, "6-yerinde-duzenle");
    h.key_press(Key::Enter);
    h.step();
    assert!(h.state().view.edit.is_none());
    let w = h.state().doc.widgets.iter().find(|w| w.name == "source_label").unwrap();
    assert_eq!(w.text, "Target folder");
}

#[test]
fn arrows_nudge_and_undo_restores() {
    let mut h = harness(Theme::Dark);
    let at = center_of(&h, "browse");
    click(&mut h, at);
    h.key_press(Key::ArrowLeft);
    h.step();
    assert_eq!(rect_of(&h, "browse")[0], 319.0);
    h.key_press_modifiers(Modifiers::SHIFT, Key::ArrowDown);
    h.step();
    assert_eq!(rect_of(&h, "browse")[1], 52.0, "Shift ile ızgara adımı");
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(rect_of(&h, "browse")[..2], [319.0, 44.0]);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(rect_of(&h, "browse")[..2], [320.0, 44.0]);
}

#[test]
fn delete_then_undo_brings_it_back() {
    let mut h = harness(Theme::Dark);
    let count = h.state().doc.widgets.len();
    let at = center_of(&h, "quality");
    click(&mut h, at);
    h.key_press(Key::Delete);
    h.step();
    assert_eq!(h.state().doc.widgets.len(), count - 1);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(h.state().doc.widgets.len(), count);
}

#[test]
fn light_theme_with_selection_and_measures() {
    let mut h = harness(Theme::Light);
    let at = center_of(&h, "quality");
    click(&mut h, at);
    h.event(Event::ModifiersChanged(Modifiers::ALT));
    h.step();
    shot(&mut h, "7-acik-tema-olculer");
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.state_mut().open_preview();
    h.run_steps(3);
}
