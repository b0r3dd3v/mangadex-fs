use crate::api;

#[derive(Debug)]
#[repr(u8)]
pub enum ViewMode {
    Detailed,
    ExpandedList,
    SimpleList,
    Grid
}

pub enum SetViewModeError {
    Request(reqwest::Error),
    Response(String)
}

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

pub async fn set_view_mode(client: &reqwest::Client, session: &api::MangaDexSession, view: ViewMode) -> Result<(), SetViewModeError> {
    let mut url = reqwest::Url::parse("https://mangadex.org/ajax/actions.ajax.php").unwrap();

    url.query_pairs_mut().append_pair("function", "set_mangas_view");
    url.query_pairs_mut().append_pair("mode", &(view as u8).to_string());

    let response = client
        .get(url)
        .headers(headers(&session))
        .send().await.map_err(SetViewModeError::Request)?
        .text().await.map_err(SetViewModeError::Request)?;
        
    let success = String::from("<div class='alert alert-success text-center' role='alert'><strong>Success:</strong> View mode set.</div>");

    info!("set view mode response: {}", response);

    if response == success { Ok(()) }
    else { Err(SetViewModeError::Response(response)) }
}