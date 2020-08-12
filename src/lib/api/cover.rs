pub struct Cover(pub Vec<u8>);

impl Cover {
    pub async fn get(client: &reqwest::Client, url: &reqwest::Url) -> Result<Cover, reqwest::Error> {
        let response = client
            .get(url.as_ref())
            .send().await?;
        
        Ok(Cover(response.bytes().await?.to_vec()))
    }
}