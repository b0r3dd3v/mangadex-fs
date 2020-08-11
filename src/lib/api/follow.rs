use crate::api;

fn headers(session: &api::MangaDexSession) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.append(
        reqwest::header::USER_AGENT,
        api::user_agent()
    );

    headers.append(
        reqwest::header::COOKIE,
        reqwest::header::HeaderValue::from_str(&format!("mangadex_session={}", session.id))
            .unwrap()
    );

    headers.append(
        reqwest::header::HeaderName::from_static("x-requested-with"),
        reqwest::header::HeaderValue::from_static("XMLHttpRequest"),
    );

    headers
}

pub async fn follow(client: &reqwest::Client, session: &api::MangaDexSession, id: u64, status: &api::MDListStatus) -> Result<(), reqwest::Error> {    
    let mut url = reqwest::Url::parse("https://mangadex.org/ajax/actions.ajax.php").unwrap();
        
    url.query_pairs_mut().append_pair("function", "manga_follow");
    url.query_pairs_mut().append_pair("id", id.to_string().as_str());
    url.query_pairs_mut().append_pair("type", status.encode().to_string().as_str());

    client
        .get(url)
        .headers(headers(session))
        .send().await.map(|_| ())
}