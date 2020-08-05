use crate::api;

#[derive(Debug, Clone)]
pub struct MangaDexSession {
    pub id: String,
    pub remember_me_token: String,
}

#[derive(Debug)]
pub enum LogOutError {
    Request(reqwest::Error),
    Response(String)
}

#[derive(Debug)]
pub enum LogInError {
    Request(reqwest::Error),
    Invalid,
    Response(String)
}

fn log_in_headers() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.append(
        reqwest::header::USER_AGENT,
        api::user_agent()
    );
    headers.append(
        reqwest::header::HeaderName::from_static("x-requested-with"),
        reqwest::header::HeaderValue::from_static("XMLHttpRequest"),
    );

    headers
}

fn log_in_form<L, P>(login: L, password: P) -> reqwest::multipart::Form
    where
        L: Into<std::borrow::Cow<'static, str>>,
        P: Into<std::borrow::Cow<'static, str>> {
    reqwest::multipart::Form::new()
        .text("login_username", login)
        .text("login_password", password)
        .text("remember_me", "1")
}

fn log_in_request(client: &reqwest::Client, headers: reqwest::header::HeaderMap, form: reqwest::multipart::Form) -> reqwest::RequestBuilder {
    client
        .post(reqwest::Url::parse("https://mangadex.org/ajax/actions.ajax.php?function=login").unwrap())
        .headers(headers)
        .multipart(form)
}

fn log_out_headers(session: &MangaDexSession) -> reqwest::header::HeaderMap {
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

fn log_out_request(client: &reqwest::Client, headers: reqwest::header::HeaderMap) -> reqwest::RequestBuilder {
    client
        .post(reqwest::Url::parse("https://mangadex.org/ajax/actions.ajax.php?function=logout").unwrap())
        .headers(headers)
}

impl MangaDexSession {
    pub async fn log_out(client: &reqwest::Client, session: &MangaDexSession) -> Result<(), LogOutError> {
        let request = log_out_request(client, log_out_headers(session));
        let result = request.send().await;

        match result {
            Ok(_) => Ok(()),
            Err(error) => Err(LogOutError::Request(error))
        }
    }

    pub async fn log_in<L, P>(
        client: &reqwest::Client,
        login: L,
        password: P,
    ) -> Result<MangaDexSession, LogInError>
    where
        L: Into<std::borrow::Cow<'static, str>>,
        P: Into<std::borrow::Cow<'static, str>> {

        let request = log_in_request(client, log_in_headers(), log_in_form(login, password));
        let result = request.send().await;

        match result {
            Ok(response) => {
                let mut mangadex_session: Option<String> = None;
                let mut mangadex_rememberme_token: Option<String> = None;
        
                for (name, value) in response
                    .headers()
                    .get_all(reqwest::header::SET_COOKIE)
                    .into_iter()
                    .map(reqwest::header::HeaderValue::to_str)
                    .map(Result::unwrap)
                    .map(cookie::Cookie::parse)
                    .map(Result::unwrap)
                    .map(|cookie| {
                        let (name, value) = cookie.name_value();
                        (name.to_owned(), value.to_owned())
                    })
                {
                    if name == "mangadex_rememberme_token" {
                        mangadex_rememberme_token = Some(value)
                    } else if name == "mangadex_session" {
                        mangadex_session = Some(value)
                    }
                }
        
                match (mangadex_session, mangadex_rememberme_token) {
                    (Some(id), Some(remember_me_token)) => Ok(MangaDexSession { id, remember_me_token }),
                    (None, None) => match response.text().await {
                        Ok(text) => {
                            let html = scraper::Html::parse_fragment(&text);
                            let selector = scraper::Selector::parse("div").unwrap();

                            match html.select(&selector).next() {
                                Some(ref element) => Err(LogInError::Response(element.text().fold(String::from(""), |acc, text| acc + text))),
                                _ => Err(LogInError::Invalid)
                            }
                        },
                        Err(error) => Err(LogInError::Request(error))
                    },
                    _ => Err(LogInError::Invalid)
                }
            }
            Err(error) => Err(LogInError::Request(error))
        }
    }
}