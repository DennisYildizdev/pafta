//! Önizleme penceresinin ve üretilen kodun ortak yardımcıları.
//!
//! Kod üretici bu dosyayı metin olarak okur ve kullanılan fonksiyonları
//! çıktıya olduğu gibi kopyalar. Önizleme de aynı fonksiyonları çağırır;
//! böylece önizlemede gördüğün, üretilen uygulamanın kendisidir.
//! Her fonksiyon `// @ad` satırıyla başlar ve yalnız `egui`ye dayanır.
//!
//! `///` yorumları çıktıya gider; bu yüzden İngilizce yazılır. Türkçe çıktıda
//! kod üretici onları `codegen::HELPER_DOCS_TR` tablosuyla çevirir.

use eframe::egui;

// @at
/// Position inside the window: (x, y) from corner `o`, size (w, h).
pub fn at(o: egui::Pos2, x: f32, y: f32, w: f32, h: f32) -> egui::Rect {
    egui::Rect::from_min_size(o + egui::vec2(x, y), egui::vec2(w, h))
}

// @label
/// Centers the text vertically and aligns it horizontally to `align`.
pub fn label(ui: &mut egui::Ui, r: egui::Rect, text: &str, align: egui::Align) -> egui::Response {
    let layout = match align {
        egui::Align::Min => egui::Layout::left_to_right(egui::Align::Center),
        egui::Align::Center => egui::Layout::centered_and_justified(egui::Direction::TopDown),
        egui::Align::Max => egui::Layout::right_to_left(egui::Align::Center),
    };
    ui.new_child(egui::UiBuilder::new().max_rect(r).layout(layout))
        .add(egui::Label::new(text).truncate())
}

// @button
pub fn button(ui: &mut egui::Ui, r: egui::Rect, text: &str) -> egui::Response {
    ui.place(r, egui::Button::new(text).min_size(r.size()))
}

// @text_field
pub fn text_field(ui: &mut egui::Ui, r: egui::Rect, value: &mut String) -> egui::Response {
    let edit = egui::TextEdit::singleline(value)
        .min_size(r.size())
        .align(egui::Align2::LEFT_CENTER);
    ui.place(r, edit)
}

// @text_area
/// Multi-line text that scrolls inside its area when it overflows.
pub fn text_area(ui: &mut egui::Ui, r: egui::Rect, id: &str, value: &mut String) -> egui::Response {
    let mut child = ui.new_child(egui::UiBuilder::new().id_salt(id).max_rect(r));
    egui::ScrollArea::vertical()
        .show(&mut child, |ui| {
            let edit = egui::TextEdit::multiline(value)
                .desired_width(f32::INFINITY)
                .desired_rows(1)
                .min_size(egui::vec2(0.0, r.height()));
            ui.add(edit)
        })
        .inner
}

// @checkbox
pub fn checkbox(ui: &mut egui::Ui, r: egui::Rect, value: &mut bool, text: &str) -> egui::Response {
    let layout = egui::Layout::left_to_right(egui::Align::Center);
    ui.new_child(egui::UiBuilder::new().max_rect(r).layout(layout))
        .checkbox(value, text)
}

// @radio
/// Option number `index` of a group; clicking it sets `value` to `index`.
pub fn radio(ui: &mut egui::Ui, r: egui::Rect, value: &mut usize, index: usize, text: &str) -> egui::Response {
    let layout = egui::Layout::left_to_right(egui::Align::Center);
    ui.new_child(egui::UiBuilder::new().max_rect(r).layout(layout))
        .radio_value(value, index, text)
}

// @combo
pub fn combo(ui: &mut egui::Ui, r: egui::Rect, id: &str, value: &mut usize, items: &[&str]) -> egui::Response {
    let layout = egui::Layout::left_to_right(egui::Align::Center);
    let mut child = ui.new_child(egui::UiBuilder::new().id_salt(id).max_rect(r).layout(layout));
    child.spacing_mut().interact_size.y = r.height();
    let selected = items.get(*value).copied().unwrap_or_default();
    egui::ComboBox::from_id_salt(id)
        .width(r.width())
        .selected_text(selected)
        .show_ui(&mut child, |ui| {
            for (i, item) in items.iter().enumerate() {
                ui.selectable_value(value, i, *item);
            }
        })
        .response
}

// @list
pub fn list(ui: &mut egui::Ui, r: egui::Rect, id: &str, value: &mut Option<usize>, items: &[&str]) -> egui::Response {
    let mut child = ui.new_child(egui::UiBuilder::new().id_salt(id).max_rect(r));
    let frame = egui::Frame::group(child.style())
        .fill(child.visuals().extreme_bg_color)
        .inner_margin(egui::Margin::same(4));
    frame
        .show(&mut child, |ui| {
            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                    for (i, item) in items.iter().enumerate() {
                        ui.selectable_value(value, Some(i), *item);
                    }
                });
            });
        })
        .response
}

// @slider
pub fn slider(ui: &mut egui::Ui, r: egui::Rect, value: &mut f32, range: std::ops::RangeInclusive<f32>) -> egui::Response {
    let layout = egui::Layout::left_to_right(egui::Align::Center);
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r).layout(layout));
    child.spacing_mut().slider_width = r.width();
    child.add(egui::Slider::new(value, range).show_value(false))
}

// @progress
pub fn progress(ui: &mut egui::Ui, r: egui::Rect, value: f32) -> egui::Response {
    let bar = egui::ProgressBar::new(value)
        .desired_width(r.width())
        .desired_height(r.height());
    ui.place(r, bar)
}

// @group
/// A thin outlined frame; the title sits on its top border.
pub fn group(ui: &mut egui::Ui, r: egui::Rect, title: &str) {
    let visuals = ui.visuals();
    let painter = ui.painter();
    let stroke = visuals.widgets.noninteractive.bg_stroke;
    painter.rect_stroke(r, 6.0, stroke, egui::StrokeKind::Inside);
    if !title.is_empty() {
        let font = egui::TextStyle::Body.resolve(ui.style());
        let galley = painter.layout_no_wrap(title.to_owned(), font, visuals.text_color());
        let pos = r.left_top() + egui::vec2(10.0, -galley.size().y / 2.0);
        let gap = egui::Rect::from_min_size(pos, galley.size()).expand2(egui::vec2(4.0, 0.0));
        painter.rect_filled(gap, 0.0, visuals.panel_fill);
        painter.galley(pos, galley, visuals.text_color());
    }
}
