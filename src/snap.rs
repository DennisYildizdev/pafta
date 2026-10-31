//! Hizalama: kenarları komşulara ve ızgaraya oturtma, ölçü çizgileri.
//! Bütün değerler tasarım pikselidir.

use eframe::egui::{Pos2, Rect, Vec2, pos2, vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Guide {
    /// `x` konumunda, `y0..y1` arasında dikey çizgi.
    Vertical { x: f32, y0: f32, y1: f32 },
    /// `y` konumunda, `x0..x1` arasında yatay çizgi.
    Horizontal { y: f32, x0: f32, x1: f32 },
}

/// Taşınan ya da boyutlanan bir dikdörtgenin oturabileceği kenarlar ve orta çizgiler.
pub struct Targets {
    rects: Vec<Rect>,
}

impl Targets {
    /// `others` yerinde kalan bileşenler, `window` pencerenin iç alanı.
    pub fn new(others: impl IntoIterator<Item = Rect>, window: Rect) -> Self {
        let mut rects: Vec<Rect> = others.into_iter().collect();
        rects.push(window);
        Self { rects }
    }

    fn xs(&self) -> impl Iterator<Item = f32> + '_ {
        self.rects.iter().flat_map(|r| [r.left(), r.center().x, r.right()])
    }

    fn ys(&self) -> impl Iterator<Item = f32> + '_ {
        self.rects.iter().flat_map(|r| [r.top(), r.center().y, r.bottom()])
    }
}

/// `values` içinden hedeflerden birine en yakın olanın farkı (hedef - değer).
fn nearest(values: &[f32], targets: impl Iterator<Item = f32>, threshold: f32) -> Option<f32> {
    let mut best: Option<f32> = None;
    for target in targets {
        for &v in values {
            let d = target - v;
            if d.abs() <= threshold && best.is_none_or(|b| d.abs() < b.abs()) {
                best = Some(d);
            }
        }
    }
    best
}

fn to_grid(v: f32, grid: Option<f32>) -> f32 {
    grid.map_or(v, |g| (v / g).round() * g)
}

pub struct Snap {
    pub offset: Vec2,
    pub guides: Vec<Guide>,
}

/// Taşınan dikdörtgen için gereken kaydırma. Yakında hedef yoksa ızgaraya oturtur.
pub fn snap_move(moving: Rect, targets: &Targets, threshold: f32, grid: Option<f32>) -> Snap {
    let dx = nearest(&[moving.left(), moving.center().x, moving.right()], targets.xs(), threshold);
    let dy = nearest(&[moving.top(), moving.center().y, moving.bottom()], targets.ys(), threshold);
    let offset = vec2(
        dx.unwrap_or_else(|| to_grid(moving.left(), grid) - moving.left()),
        dy.unwrap_or_else(|| to_grid(moving.top(), grid) - moving.top()),
    );
    let snapped = moving.translate(offset);
    let mut guides = Vec::new();
    if dx.is_some() {
        guides.extend(vertical_guides(snapped, targets));
    }
    if dy.is_some() {
        guides.extend(horizontal_guides(snapped, targets));
    }
    Snap { offset, guides }
}

/// Tek bir dikey kenarı (sol ya da sağ) oturtur.
pub fn snap_x(x: f32, targets: &Targets, threshold: f32, grid: Option<f32>) -> f32 {
    nearest(&[x], targets.xs(), threshold).map_or_else(|| to_grid(x, grid), |d| x + d)
}

/// Tek bir yatay kenarı (üst ya da alt) oturtur.
pub fn snap_y(y: f32, targets: &Targets, threshold: f32, grid: Option<f32>) -> f32 {
    nearest(&[y], targets.ys(), threshold).map_or_else(|| to_grid(y, grid), |d| y + d)
}

/// Dikdörtgenin hedeflerle tam çakışan kenarları için çizgiler.
pub fn guides_for(r: Rect, targets: &Targets) -> Vec<Guide> {
    let mut guides = vertical_guides(r, targets);
    guides.extend(horizontal_guides(r, targets));
    guides
}

fn vertical_guides(r: Rect, targets: &Targets) -> Vec<Guide> {
    let mut out = Vec::new();
    for x in [r.left(), r.center().x, r.right()] {
        let (mut y0, mut y1, mut hit) = (r.top(), r.bottom(), false);
        for t in &targets.rects {
            if [t.left(), t.center().x, t.right()].iter().any(|tx| (tx - x).abs() < 0.5) {
                y0 = y0.min(t.top());
                y1 = y1.max(t.bottom());
                hit = true;
            }
        }
        if hit {
            out.push(Guide::Vertical { x, y0, y1 });
        }
    }
    out
}

fn horizontal_guides(r: Rect, targets: &Targets) -> Vec<Guide> {
    let mut out = Vec::new();
    for y in [r.top(), r.center().y, r.bottom()] {
        let (mut x0, mut x1, mut hit) = (r.left(), r.right(), false);
        for t in &targets.rects {
            if [t.top(), t.center().y, t.bottom()].iter().any(|ty| (ty - y).abs() < 0.5) {
                x0 = x0.min(t.left());
                x1 = x1.max(t.right());
                hit = true;
            }
        }
        if hit {
            out.push(Guide::Horizontal { y, x0, x1 });
        }
    }
    out
}

/// Bir ölçü çizgisi: dikdörtgenden en yakın komşuya ya da kapsayan çerçeveye.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measure {
    pub from: Pos2,
    pub to: Pos2,
    pub value: f32,
}

/// Dört yöndeki uzaklıklar. Komşu yoksa onu kapsayan çerçeveye ya da pencere kenarına ölçer.
pub fn measures(t: Rect, others: &[Rect], window: Rect) -> Vec<Measure> {
    let (mut left, mut right, mut top, mut bottom) =
        (window.left(), window.right(), window.top(), window.bottom());
    for o in others {
        if o.contains_rect(t) {
            left = left.max(o.left());
            right = right.min(o.right());
            top = top.max(o.top());
            bottom = bottom.min(o.bottom());
            continue;
        }
        let beside = o.top() < t.bottom() && o.bottom() > t.top();
        let stacked = o.left() < t.right() && o.right() > t.left();
        if beside && o.right() <= t.left() {
            left = left.max(o.right());
        }
        if beside && o.left() >= t.right() {
            right = right.min(o.left());
        }
        if stacked && o.bottom() <= t.top() {
            top = top.max(o.bottom());
        }
        if stacked && o.top() >= t.bottom() {
            bottom = bottom.min(o.top());
        }
    }
    let (cx, cy) = (t.center().x, t.center().y);
    [
        (pos2(left, cy), pos2(t.left(), cy), t.left() - left),
        (pos2(t.right(), cy), pos2(right, cy), right - t.right()),
        (pos2(cx, top), pos2(cx, t.top()), t.top() - top),
        (pos2(cx, t.bottom()), pos2(cx, bottom), bottom - t.bottom()),
    ]
    .into_iter()
    .filter(|(_, _, v)| *v >= 0.5)
    .map(|(from, to, value)| Measure { from, to, value })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(pos2(x, y), vec2(w, h))
    }

    #[test]
    fn moving_rect_snaps_to_neighbour_edge() {
        let window = rect(0.0, 0.0, 400.0, 300.0);
        let targets = Targets::new([rect(100.0, 20.0, 80.0, 28.0)], window);
        let snap = snap_move(rect(103.0, 61.0, 80.0, 28.0), &targets, 6.0, Some(8.0));
        assert_eq!(snap.offset.x, -3.0, "sol kenar komşunun sol kenarına oturur");
        assert_eq!(snap.offset.y, 3.0, "yakın hedef yoksa ızgaraya oturur");
        assert!(snap.guides.contains(&Guide::Vertical { x: 100.0, y0: 20.0, y1: 92.0 }));
    }

    #[test]
    fn measures_reach_neighbours_and_frames() {
        let window = rect(0.0, 0.0, 400.0, 300.0);
        let frame = rect(20.0, 20.0, 300.0, 200.0);
        let left = rect(40.0, 50.0, 60.0, 20.0);
        let list = measures(rect(116.0, 50.0, 80.0, 20.0), &[frame, left], window);
        let values: Vec<f32> = list.iter().map(|m| m.value).collect();
        assert_eq!(values, [16.0, 124.0, 30.0, 150.0]);
    }
}
