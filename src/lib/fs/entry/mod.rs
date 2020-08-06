pub mod chapter;
pub mod manga;
pub mod page;

pub use chapter::*;
pub use manga::*;
pub use page::*;

pub enum Entry {
    Manga(Manga),
    Chapter(Chapter),
    Page(Page)
}