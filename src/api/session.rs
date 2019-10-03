use reqwest::header::*;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct MangadexSession {
    pub id: String,
    pub remember_me_token: String,
}

impl MangadexSession {
    pub fn login<S: Into<String>>(
        client: &reqwest::Client,
        login: S,
        password: S,
    ) -> Result<MangadexSession, Box<dyn Error>> {
        let headers = {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.append(
                REFERER,
                HeaderValue::from_static("https://mangadex.org/login"),
            );
            headers.append(USER_AGENT, HeaderValue::from_static("urmomgay"));
            headers.append(
                HeaderName::from_static("x-requested-with"),
                HeaderValue::from_static("XMLHttpRequest"),
            );

            headers
        };

        let form = reqwest::multipart::Form::new()
            .text("login_username", login.into())
            .text("login_password", password.into())
            .text("remember_me", "1");

        let response = client
            .post(
                reqwest::Url::parse("https://mangadex.org/ajax/actions.ajax.php?function=login")
                    .unwrap(),
            )
            .headers(headers)
            .multipart(form)
            .send()?;

        let mut mangadex_session: Option<String> = None;
        let mut mangadex_rememberme_token: Option<String> = None;

        for (name, value) in response
            .headers()
            .get_all(SET_COOKIE)
            .into_iter()
            .map(|wat| wat.to_str().ok())
            .map(Option::unwrap)
            .map(cookie::Cookie::parse)
            .map(Result::unwrap)
            .map(|x| {
                let (name, value) = x.name_value();
                (name.to_owned(), value.to_owned())
            })
        {
            if name == "mangadex_rememberme_token" {
                mangadex_rememberme_token = Some(value)
            } else if name == "mangadex_session" {
                mangadex_session = Some(value)
            }
        }

        if mangadex_session.is_some() && mangadex_rememberme_token.is_some() {
            Ok(MangadexSession {
                id: mangadex_session.unwrap(),
                remember_me_token: mangadex_rememberme_token.unwrap(),
            })
        } else {
            Err("Login not successful.".into())
        }
    }
}
