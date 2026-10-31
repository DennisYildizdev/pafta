//! Arayüz dili. Varsayılan İngilizce; Görünüm → Dil menüsünden Türkçe seçilir.
//!
//! Metinler kullanıldıkları yerde iki dilde yazılır: `lang.pick("File", "Dosya")`.
//! Böylece bir metnin karşılığı unutulamaz; ikisi de derleme anında zorunludur.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lang {
    #[default]
    En,
    Tr,
}

impl Lang {
    /// İki karşılıktan geçerli dilinkini seçer.
    pub fn pick<T>(self, en: T, tr: T) -> T {
        match self {
            Lang::En => en,
            Lang::Tr => tr,
        }
    }
}
