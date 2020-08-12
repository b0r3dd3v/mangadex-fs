pub struct Page {
    pub chapter_id: u64,
    pub data: Vec<u8>
}

impl Page {
    pub async fn get(client: &reqwest::Client, chapter_id: u64, url: &reqwest::Url) -> Result<Page, reqwest::Error> {
        let response = client
            .get(url.as_ref())
            .send().await?;
        
        Ok(Page { chapter_id: chapter_id, data: response.bytes().await?.to_vec() })
    }
}