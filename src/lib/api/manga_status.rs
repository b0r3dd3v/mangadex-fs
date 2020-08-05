#[derive(Debug, PartialEq, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum MangaStatus {
    OnGoing = 1,
    Completed = 2,
    Cancelled = 3,
    Hiatus = 4
}

impl std::convert::TryFrom<u8> for MangaStatus {
    type Error = ();
    
    fn try_from(id: u8) -> Result<Self, Self::Error> {
        match id {
            1u8 => Ok(MangaStatus::OnGoing),
            2u8 => Ok(MangaStatus::Completed),
            3u8 => Ok(MangaStatus::Cancelled),
            4u8 => Ok(MangaStatus::Hiatus),
            _ => Err(())
        }
    }
}