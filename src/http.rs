use reqwest::header::{CONTENT_TYPE, HeaderMap};
use serde::de::DeserializeOwned;

use crate::error::{Error, Result};

#[derive(Clone, Debug, Default)]
pub struct HttpClient {
    inner: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
        }
    }

    pub async fn send_json<T: DeserializeOwned>(
        &self,
        url: &str,
        method: reqwest::Method,
        headers: Option<HeaderMap>,
        query: Option<&[(&str, &str)]>,
        body: Option<String>,
    ) -> Result<T> {
        let mut request = self.inner.request(method, url);
        if let Some(headers) = headers {
            request = request.headers(headers);
        }
        if let Some(query) = query {
            request = request.query(query);
        }
        if let Some(body) = body {
            request = request.header(CONTENT_TYPE, "application/json").body(body);
        }

        let response = request.send().await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Api { status, body });
        }

        Ok(response.json().await?)
    }
}
