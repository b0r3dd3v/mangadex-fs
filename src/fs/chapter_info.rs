#[derive(Debug, Clone)]
pub struct ChapterInfo {
    pub id: u64,
    pub chapter: String,
    pub volume: String,
    pub title: String,
}

impl ChapterInfo {
    pub fn format(&self) -> String {
        if self.title.is_empty() {
            if self.volume.is_empty() {
                format!("{}", self.chapter)
            }
            else {
                format!("{}.{}", self.volume, self.chapter)
            }
        } else {
            if !self.volume.is_empty() {
                format!("{}.{} {}", self.volume, self.chapter, self.title)
            }
            else {
                format!("{} {}", self.chapter, self.title)
            }
        }
    }
}