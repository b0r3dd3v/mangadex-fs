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
        if self.title.is_empty() {
            if self.volume.is_empty() {
                sanitize(format!("{} [{:06x}]", self.chapter, self.get_hash()))
            } else {
                sanitize(format!(
                    "{}.{} [{:06x}]",
                    self.volume,
                    self.chapter,
                    self.get_hash()
                ))
            }
        } else {
            if !self.volume.is_empty() {
                sanitize(format!(
                    "{}.{} {} [{:06x}]",
                    self.volume,
                    self.chapter,
                    self.title,
                    self.get_hash()
                ))
            } else {
                sanitize(format!(
                    "{} {} [{:06x}]",
                    self.chapter,
                    self.title,
                    self.get_hash()
                ))
            }
        }
    }
}
