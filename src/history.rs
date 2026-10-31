//! Geri al ve yinele: belgenin anlık görüntüleri.
//!
//! Her değişiklik ayrı ayrı kaydedilmez. Fare bırakılıp yazı alanından
//! çıkılınca, son kayıttan farklıysa yeni bir adım açılır. Böylece bir
//! sürükleme ya da bir ad yazımı tek adımda geri alınır.

use crate::model::Document;

const LIMIT: usize = 200;

pub struct History {
    undo: Vec<Document>,
    redo: Vec<Document>,
    /// Son kaydedilen hâl.
    stable: Document,
}

impl History {
    pub fn new(doc: &Document) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            stable: doc.clone(),
        }
    }

    /// Belge son kayıttan farklıysa yeni bir adım açar.
    pub fn commit(&mut self, doc: &Document) {
        if *doc != self.stable {
            self.undo.push(std::mem::replace(&mut self.stable, doc.clone()));
            if self.undo.len() > LIMIT {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
    }

    pub fn undo(&mut self, doc: &mut Document) -> bool {
        self.commit(doc);
        let Some(prev) = self.undo.pop() else {
            return false;
        };
        self.redo.push(std::mem::replace(&mut self.stable, prev.clone()));
        *doc = prev;
        true
    }

    pub fn redo(&mut self, doc: &mut Document) -> bool {
        self.commit(doc);
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.undo.push(std::mem::replace(&mut self.stable, next.clone()));
        *doc = next;
        true
    }

    pub fn can_undo(&self, doc: &Document) -> bool {
        !self.undo.is_empty() || *doc != self.stable
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Yeni ya da açılan belgeyle geçmişi sıfırlar.
    pub fn reset(&mut self, doc: &Document) {
        self.undo.clear();
        self.redo.clear();
        self.stable = doc.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_and_redo_walk_the_steps() {
        let mut doc = Document::default();
        let mut history = History::new(&doc);
        doc.title = "Bir".into();
        history.commit(&doc);
        doc.title = "İki".into();
        assert!(history.undo(&mut doc), "yazılmamış değişiklik de geri alınır");
        assert_eq!(doc.title, "Bir");
        assert!(history.undo(&mut doc));
        assert_eq!(doc.title, "New window");
        assert!(!history.undo(&mut doc));
        assert!(history.redo(&mut doc));
        assert!(history.redo(&mut doc));
        assert_eq!(doc.title, "İki");
        assert!(!history.can_redo());
    }
}
