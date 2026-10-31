# Pafta

egui arayüzlerini elle tasarlama aracı. Bileşenleri bir pencereye sürükleyip yerleştirirsin;
Pafta tasarımı `cargo run` ile çalışan tek dosyalık bir eframe uygulamasına çevirir.

```
cargo run --release
```

## Tek dosya exe

```
cargo build --release
```

`target/release/pafta.exe` tek başına taşınabilir; kolay bulunsun diye bir kopyası
`dist/Pafta.exe` olarak durur. Simgesi ve sürüm bilgisi exe'ye gömülüdür (`build.rs`),
Visual C++ çalışma zamanı da içindedir (`.cargo/config.toml`). Başka bir Windows 10/11
bilgisayarda kurulum gerekmeden açılır.

## Dil

Arayüz İngilizce açılır. Türkçe için: View → Language → Türkçe (Görünüm → Dil). Seçim saklanır.
Dil; menüleri, yeni bileşenlerin adlarını (`button_1` / `dugme_1`), kısayol harflerini ve üretilen
koddaki yorumları belirler. Tasarımdaki metinlere dokunmaz.

## Nasıl çalışır

- **Palet** (sol): bir bileşeni tuvale sürükle, ya da tıklayıp tuvalde çiz. Harf kısayolları
  satırın sağında yazar (İngilizcede T label, B button…; Türkçede E etiket, D düğme…).
- **Tuval** (orta): pencere tek renkli bir pafta gibi çizilir. Taşırken komşu kenarlara ve
  ortalara oturur (pembe çizgiler), uzaklıklar ölçü çizgisiyle görünür. Alt basılıyken
  yaslama kapanır ve ölçüler görünür. Pencerenin sağ alt köşesini sürükleyerek boyutunu değiştir.
- **Özellikler** (sağ): ad, metin, öğeler, konum ve boyut; çoklu seçimde hizalama ve dağıtma.
- **Kod** (alt): tasarım değiştikçe `main.rs` yeniden yazılır. Ctrl+E ile kaydet ya da
  Dosya → Cargo projesi olarak dışa aktar ile boş bir klasöre çalışan bir proje çıkar.
- **Önizleme** (F5): tasarımı ayrı bir pencerede, üretilen kodla aynı egui bileşenleriyle çalıştırır.

Tasarımlar `.pafta` uzantılı JSON dosyalarıdır. Pafta kapanırken kaldığın yeri saklar.

## Kısayollar

| İş | Tuş |
| --- | --- |
| Bileşen aracı (İngilizce) | T B F A C R D L S P G |
| Bileşen aracı (Türkçe) | E D G M O S A L K I C |
| Metni yerinde düzenle | Enter ya da çift tık |
| Taşı | Oklar (Shift: ızgara adımı) |
| Çoğalt, sil | Ctrl+D, Delete |
| Geri al, yinele | Ctrl+Z, Ctrl+Y |
| Öne al, arkaya al | PgUp, PgDn (Shift: en uca) |
| Yakınlaştır | Ctrl + tekerlek, Ctrl+0 gerçek boyut |
| Kaydır | Tekerlek, Boşluk + sürükle |
| Kod paneli, önizleme | Ctrl+J, F5 |

Hepsi uygulamada F1 ile görünür.

## Geliştirme

```
cargo test
```

Arayüz testleri tuvali sanal fareyle sürer (`src/ui_tests.rs`). Kareleri görmek için:
`PAFTA_GORUNTU=klasör cargo test`. Örnek tasarımın kodunu yazdırmak için:
`cargo run --example ornek_kod` (Türkçe çıktı için sona `-- tr`).

`src/live.rs` hem önizlemeyi çalıştırır hem de kod üreticinin çıktıya kopyaladığı
yardımcıları tutar; önizlemede görünen, üretilen uygulamanın kendisidir.
