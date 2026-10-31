//! Windows'ta exe'ye simge ve dosya bilgisi gömer. Simge `src/icon.rs` çiziminden üretilir.

#[path = "src/icon.rs"]
mod icon;

fn main() {
    println!("cargo:rerun-if-changed=src/icon.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR yok"));
    let ico = out.join("pafta.ico");
    std::fs::write(&ico, ico_file(&[16, 24, 32, 48, 64, 256])).expect("simge yazılamadı");

    let mut res = winresource::WindowsResource::new();
    res.set_icon(ico.to_str().expect("OUT_DIR yolu UTF-8 değil"));
    // Windows, Görev Yöneticisi gibi yerlerde uygulama adı olarak FileDescription'ı gösterir.
    res.set("ProductName", "Pafta");
    res.set("FileDescription", "Pafta");
    res.set("Comments", "egui arayüzlerini elle tasarlama aracı");
    res.compile().expect("Windows kaynağı derlenemedi");
}

/// Her boyut için 32 bit BMP kaydı olan bir .ico dosyası.
fn ico_file(sizes: &[u32]) -> Vec<u8> {
    let images: Vec<Vec<u8>> = sizes.iter().map(|&size| dib(size)).collect();
    let mut out = Vec::new();
    for v in [0u16, 1, sizes.len() as u16] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (&size, image) in sizes.iter().zip(&images) {
        // 256 piksel, tek baytlık alana 0 olarak yazılır.
        let side = if size >= 256 { 0 } else { size as u8 };
        out.extend_from_slice(&[side, side, 0, 0]);
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(&(image.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += image.len() as u32;
    }
    for image in images {
        out.extend(image);
    }
    out
}

/// BITMAPINFOHEADER, alttan üste BGRA pikseller, ardından saydamlık (AND) maskesi.
fn dib(size: u32) -> Vec<u8> {
    let rgba = icon::rgba(size);
    let n = size as usize;
    let mask_row = n.div_ceil(32) * 4;
    let mut out = Vec::new();
    // Yükseklik iki katı yazılır: piksel verisi ve maske.
    for v in [40u32, size, size * 2] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&((n * n * 4 + mask_row * n) as u32).to_le_bytes());
    out.extend_from_slice(&[0u8; 16]);
    for y in (0..n).rev() {
        for x in 0..n {
            let i = (y * n + x) * 4;
            out.extend_from_slice(&[rgba[i + 2], rgba[i + 1], rgba[i], rgba[i + 3]]);
        }
    }
    for y in (0..n).rev() {
        let mut row = vec![0u8; mask_row];
        for x in 0..n {
            if rgba[(y * n + x) * 4 + 3] < 128 {
                row[x / 8] |= 0x80 >> (x % 8);
            }
        }
        out.extend(row);
    }
    out
}
