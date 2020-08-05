pub type Response = u8;

pub const LOG_IN_RESULT: Response = 1u8;
pub const LOG_OUT_RESULT: Response = 2u8;
pub const ADD_MANGA_RESULT: Response = 3u8;
pub const ADD_CHAPTER_RESULT: Response = 4u8;
pub const QUICK_SEARCH_RESULT: Response = 5u8;

pub type LogInResult = u8;
pub const LOG_IN_RESULT_OK: LogInResult = 0u8; 
pub const LOG_IN_RESULT_ERROR_REQUEST: LogInResult = 1u8;
pub const LOG_IN_RESULT_ERROR_RESPONSE: LogInResult = 2u8;
pub const LOG_IN_RESULT_ERROR_INVALID: LogInResult = 3u8;

pub type LogOutResult = u8;
pub const LOG_OUT_RESULT_OK: LogInResult = 0u8; 
pub const LOG_OUT_RESULT_ERROR_REQUEST: LogInResult = 1u8;
pub const LOG_OUT_RESULT_ERROR_RESPONSE: LogInResult = 2u8;

pub type AddMangaResult = u8;
pub const ADD_MANGA_RESULT_OK: AddMangaResult = 0u8;
pub const ADD_MANGA_RESULT_ERROR_REQUEST: AddMangaResult = 1u8;

pub type AddChapterResult = u8;
pub const ADD_CHAPTER_RESULT_OK: AddChapterResult = 0u8;
pub const ADD_CHAPTER_RESULT_ERROR_REQUEST: AddChapterResult = 1u8;

pub type QuickSearchResult = u8;
pub const QUICK_SEARCH_RESULT_OK: QuickSearchResult = 0u8; 
pub const QUICK_SEARCH_RESULT_ERROR_REQUEST: QuickSearchResult = 1u8;
pub const QUICK_SEARCH_RESULT_ERROR_NOT_LOGGED_IN: QuickSearchResult = 2u8;