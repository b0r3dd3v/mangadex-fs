pub fn user_agent() -> reqwest::header::HeaderValue {
    reqwest::header::HeaderValue::from_str(
        format!("{}/{} (+{})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_REPOSITORY")
        ).as_str()
    ).unwrap()
}