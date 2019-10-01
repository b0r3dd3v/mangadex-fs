use sanitize_filename::sanitize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct ChapterInfo {
    pub id: u64,
    pub chapter: String,
    pub volume: String,
    pub title: String,
}

impl ChapterInfo {
    pub fn get_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
}

impl Hash for ChapterInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl ChapterInfo {
    pub fn format(&self) -> String {
        let hash = self.get_hash();

        match (self.title.is_empty(), self.volume.is_empty()) {
            (true, true) => sanitize(format!("{} [{:06x}]", self.chapter, hash)),
            (true, false) => sanitize(format!(
                "{}.{} [{:06x}]",
                self.volume,
                self.chapter,
                hash
            )),
            (false, true) => sanitize(format!(
                "{} {} [{:06x}]",
                self.chapter,
                self.title,
                hash
            )),
            _ => sanitize(format!(
                "{}.{} {} [{:06x}]",
                self.volume,
                self.chapter,
                self.title,
                hash
            )),
        }
    }
}
