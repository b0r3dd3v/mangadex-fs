use crate::api;

#[derive(Debug)]
pub enum Page {
    Ready(Vec<u8>),
    Proxy(usize)
}

impl Page {
    pub fn proxy(page_api: api::PageProxy) -> Page {
        Page::Proxy(page_api.size)
    }

    pub fn ready(page_api: api::Page) -> Page {
        Page::Ready(page_api.data)
    }
    
    pub fn size(&self) -> usize {
        match &self {
            Page::Proxy(size) => *size,
            Page::Ready(data) => data.len()
        }
    }
}