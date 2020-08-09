pub struct PageProxy {
    pub size: usize
}

pub struct Page {
    pub chapter_id: u64,
    pub data: Vec<u8>
}

impl PageProxy {
    pub async fn get(client: &reqwest::Client, url: &reqwest::Url) -> Result<PageProxy, reqwest::Error> {
        let response = client.head(url.as_ref()).send().await?;
        let headers = response.headers();
        let content_length = &headers[reqwest::header::CONTENT_LENGTH];

        let size = content_length.to_str().unwrap().parse::<usize>().unwrap();

        Ok(PageProxy { size })
    }
}

impl Page {
    pub async fn get(client: &reqwest::Client, chapter_id: u64, url: &reqwest::Url) -> Result<Page, reqwest::Error> {
        let response = client
            .get(url.as_ref())
            .send().await?;
        
        Ok(Page { chapter_id: chapter_id, data: response.bytes().await?.to_vec() })
    }
}