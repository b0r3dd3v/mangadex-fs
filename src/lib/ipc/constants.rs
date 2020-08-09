pub const COMMAND_END_CONNECTION: u8 = 0u8;
pub const COMMAND_KILL: u8 = 1u8;
pub const COMMAND_LOG_IN: u8 = 2u8;
pub const COMMAND_LOG_OUT: u8 = 3u8;
pub const COMMAND_ADD_MANGA: u8 = 4u8;
pub const COMMAND_SEARCH: u8 = 5u8;

pub const RESPONSE_KILL: u8 = 1u8;
pub const RESPONSE_LOG_IN: u8 = 2u8;
pub const RESPONSE_LOG_OUT: u8 = 3u8;
pub const RESPONSE_ADD_MANGA: u8 = 4u8;
pub const RESPONSE_SEARCH: u8 = 5u8;

pub const RESULT_OK: u8 = 0u8;
pub const RESULT_ERR: u8 = 1u8;

pub const OPTION_SOME: u8 = 0u8;
pub const OPTION_NONE: u8 = 1u8;