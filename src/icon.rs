//! Uygulama simgesi: koyu, yuvarlak köşeli karede bir pafta; sağ alt köşesinde mavi antet.
//!
//! Pencere simgesi de, build.rs'in exe'ye gömdüğü .ico da bu çizimden çıkar.
//! build.rs bu dosyayı doğrudan derlediği için yalnız standart kütüphaneye dayanır.

/// `size` × `size` RGBA pikseller, üst satırdan başlayarak.
pub fn rgba(size: u32) -> Vec<u8> {
    // Piksel başına 4×4 örnek: 16 piksellik simgede de kenarlar yumuşak kalır.
    const SAMPLES: u32 = 4;
    let n = size as usize;
    let scale = 64.0 / size as f32;
    let mut out = vec![0u8; n * n * 4];
    for y in 0..size {
        for x in 0..size {
            let (mut sum, mut hits) = ([0.0f32; 3], 0.0f32);
            for sy in 0..SAMPLES {
                for sx in 0..SAMPLES {
                    let px = (x as f32 + (sx as f32 + 0.5) / SAMPLES as f32) * scale;
                    let py = (y as f32 + (sy as f32 + 0.5) / SAMPLES as f32) * scale;
                    if let Some(rgb) = color_at(px, py) {
                        for (s, c) in sum.iter_mut().zip(rgb) {
                            *s += f32::from(c);
                        }
                        hits += 1.0;
                    }
                }
            }
            if hits > 0.0 {
                let i = (y as usize * n + x as usize) * 4;
                for (o, s) in out[i..i + 3].iter_mut().zip(sum) {
                    *o = (s / hits).round() as u8;
                }
                out[i + 3] = (hits / (SAMPLES * SAMPLES) as f32 * 255.0).round() as u8;
            }
        }
    }
    out
}

/// 64 birimlik çizim alanında bir noktanın rengi; simgenin dışındaysa `None`.
fn color_at(x: f32, y: f32) -> Option<[u8; 3]> {
    const RADIUS: f32 = 14.0;
    let (cx, cy) = (x.clamp(RADIUS, 64.0 - RADIUS), y.clamp(RADIUS, 64.0 - RADIUS));
    if (x - cx).powi(2) + (y - cy).powi(2) > RADIUS * RADIUS {
        return None;
    }
    let within = |from: f32, to: f32| (from..to).contains(&x) && (from..to).contains(&y);
    if (31.0..50.0).contains(&x) && (37.0..50.0).contains(&y) {
        Some([0x0a, 0x84, 0xff])
    } else if within(14.0, 50.0) && !within(16.0, 48.0) {
        Some([0xf5, 0xf5, 0xf7])
    } else {
        Some([0x1d, 0x1d, 0x1f])
    }
}
