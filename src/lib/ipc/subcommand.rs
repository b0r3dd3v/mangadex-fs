pub type SubCommand = u8;

pub const KILL: SubCommand = 0u8;
pub const LOG_IN: SubCommand = 1u8;
pub const LOG_OUT: SubCommand = 2u8;
pub const ADD_MANGA: SubCommand = 3u8;
pub const ADD_CHAPTER: SubCommand = 4u8;
pub const QUICK_SEARCH: SubCommand = 5u8;