pub const COMMAND_END_CONNECTION: u8 = 0u8;
pub const COMMAND_KILL: u8 = 1u8;
pub const COMMAND_LOG_IN: u8 = 2u8;
pub const COMMAND_LOG_OUT: u8 = 3u8;
pub const COMMAND_ADD_MANGA: u8 = 4u8;
pub const COMMAND_SEARCH: u8 = 5u8;
pub const COMMAND_MDLIST: u8 = 6u8;
pub const COMMAND_FOLLOW_MANGA: u8 = 7u8;
pub const COMMAND_UNFOLLOW_MANGA: u8 = 8u8;

pub const RESPONSE_KILL: u8 = 1u8;
pub const RESPONSE_LOG_IN: u8 = 2u8;
pub const RESPONSE_LOG_OUT: u8 = 3u8;
pub const RESPONSE_ADD_MANGA: u8 = 4u8;
pub const RESPONSE_SEARCH: u8 = 5u8;
pub const RESPONSE_MDLIST: u8 = 6u8;
pub const RESPONSE_FOLLOW_MANGA: u8 = 7u8;
pub const RESPONSE_UNFOLLOW_MANGA: u8 = 8u8;

pub const RESULT_OK: u8 = 0u8;
pub const RESULT_ERR: u8 = 1u8;

pub const OPTION_SOME: u8 = 0u8;
pub const OPTION_NONE: u8 = 1u8;

pub const MDLIST_STATUS_READING: u8 = 0u8;
pub const MDLIST_STATUS_COMPLETED: u8 = 1u8;
pub const MDLIST_STATUS_ON_HOLD: u8 = 2u8;
pub const MDLIST_STATUS_PLAN_TO_READ: u8 = 3u8;
pub const MDLIST_STATUS_DROPPED: u8 = 4u8;
pub const MDLIST_STATUS_RE_READING: u8 = 5u8;