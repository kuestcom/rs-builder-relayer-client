use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD, URL_SAFE};
use hmac::{Hmac, KeyInit as _, Mac as _};
use reqwest::header::HeaderMap;
use secrecy::{ExposeSecret as _, SecretString};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use url::Url;

use crate::error::{Error, Result};

pub const KUEST_BUILDER_API_KEY: &str = "KUEST_BUILDER_API_KEY";
pub const KUEST_BUILDER_PASSPHRASE: &str = "KUEST_BUILDER_PASSPHRASE";
pub const KUEST_BUILDER_SIGNATURE: &str = "KUEST_BUILDER_SIGNATURE";
pub const KUEST_BUILDER_TIMESTAMP: &str = "KUEST_BUILDER_TIMESTAMP";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuilderType {
    Unavailable,
    Local,
    Remote,
}

#[derive(Clone)]
pub struct BuilderApiKeyCreds {
    pub key: String,
    pub secret: SecretString,
    pub passphrase: SecretString,
}

impl std::fmt::Debug for BuilderApiKeyCreds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuilderApiKeyCreds")
            .field("key", &"<redacted>")
            .field("secret", &"<redacted>")
            .field("passphrase", &"<redacted>")
            .finish()
    }
}

impl BuilderApiKeyCreds {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        secret: impl Into<String>,
        passphrase: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            secret: SecretString::from(secret.into()),
            passphrase: SecretString::from(passphrase.into()),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct RemoteBuilderConfig {
    pub url: Url,
    pub token: Option<String>,
}

impl std::fmt::Debug for RemoteBuilderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token = self.token.as_ref().map(|_| "<redacted>");

        f.debug_struct("RemoteBuilderConfig")
            .field("url", &self.url)
            .field("token", &token)
            .finish()
    }
}

impl RemoteBuilderConfig {
    pub fn new(url: &str, token: Option<String>) -> Result<Self> {
        let url = Url::parse(url)?;
        if url.scheme() != "http" && url.scheme() != "https" {
            return Err(Error::InvalidRemoteUrl);
        }
        if matches!(token.as_deref(), Some("")) {
            return Err(Error::InvalidAuthToken);
        }

        Ok(Self { url, token })
    }
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BuilderHeaderPayload {
    #[serde(rename = "KUEST_BUILDER_API_KEY")]
    pub kuest_builder_api_key: String,
    #[serde(rename = "KUEST_BUILDER_TIMESTAMP")]
    pub kuest_builder_timestamp: String,
    #[serde(rename = "KUEST_BUILDER_PASSPHRASE")]
    pub kuest_builder_passphrase: String,
    #[serde(rename = "KUEST_BUILDER_SIGNATURE")]
    pub kuest_builder_signature: String,
}

impl std::fmt::Debug for BuilderHeaderPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuilderHeaderPayload")
            .field("kuest_builder_api_key", &"<redacted>")
            .field("kuest_builder_timestamp", &self.kuest_builder_timestamp)
            .field("kuest_builder_passphrase", &"<redacted>")
            .field("kuest_builder_signature", &"<redacted>")
            .finish()
    }
}

impl BuilderHeaderPayload {
    pub fn to_header_map(&self) -> Result<HeaderMap> {
        let mut map = HeaderMap::new();
        map.insert(KUEST_BUILDER_API_KEY, self.kuest_builder_api_key.parse()?);
        map.insert(
            KUEST_BUILDER_PASSPHRASE,
            self.kuest_builder_passphrase.parse()?,
        );
        map.insert(
            KUEST_BUILDER_SIGNATURE,
            self.kuest_builder_signature.parse()?,
        );
        map.insert(
            KUEST_BUILDER_TIMESTAMP,
            self.kuest_builder_timestamp.parse()?,
        );
        Ok(map)
    }
}

#[derive(Clone, Debug, Default)]
pub struct BuilderConfig {
    remote_builder_config: Option<RemoteBuilderConfig>,
    local_builder_creds: Option<BuilderApiKeyCreds>,
    http_client: reqwest::Client,
}

impl BuilderConfig {
    pub fn from_parts(
        remote_builder_config: Option<RemoteBuilderConfig>,
        local_builder_creds: Option<BuilderApiKeyCreds>,
    ) -> Result<Self> {
        if let Some(remote) = &remote_builder_config {
            if remote.url.scheme() != "http" && remote.url.scheme() != "https" {
                return Err(Error::InvalidRemoteUrl);
            }
            if matches!(remote.token.as_deref(), Some("")) {
                return Err(Error::InvalidAuthToken);
            }
        }

        if let Some(local) = &local_builder_creds
            && !has_valid_local_creds(local)
        {
            return Err(Error::InvalidLocalBuilderCredentials);
        }

        Ok(Self {
            remote_builder_config,
            local_builder_creds,
            http_client: reqwest::Client::new(),
        })
    }

    pub fn local(local_builder_creds: BuilderApiKeyCreds) -> Result<Self> {
        Self::from_parts(None, Some(local_builder_creds))
    }

    pub fn remote(url: &str, token: Option<String>) -> Result<Self> {
        Self::from_parts(Some(RemoteBuilderConfig::new(url, token)?), None)
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.get_builder_type() != BuilderType::Unavailable
    }

    #[must_use]
    pub fn get_builder_type(&self) -> BuilderType {
        if self.local_builder_creds.is_some() {
            BuilderType::Local
        } else if self.remote_builder_config.is_some() {
            BuilderType::Remote
        } else {
            BuilderType::Unavailable
        }
    }

    pub async fn generate_builder_headers(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        timestamp: Option<u64>,
    ) -> Result<BuilderHeaderPayload> {
        match self.get_builder_type() {
            BuilderType::Local => {
                let creds = self
                    .local_builder_creds
                    .as_ref()
                    .ok_or(Error::BuilderCredentialsUnavailable)?;
                let ts = timestamp.unwrap_or_else(current_unix_timestamp);
                let signature =
                    build_hmac_signature(creds.secret.expose_secret(), ts, method, path, body)?;
                Ok(BuilderHeaderPayload {
                    kuest_builder_api_key: creds.key.clone(),
                    kuest_builder_timestamp: ts.to_string(),
                    kuest_builder_passphrase: creds.passphrase.expose_secret().to_owned(),
                    kuest_builder_signature: signature,
                })
            }
            BuilderType::Remote => {
                let remote = self
                    .remote_builder_config
                    .as_ref()
                    .ok_or(Error::BuilderCredentialsUnavailable)?;
                let payload = serde_json::json!({
                    "method": method,
                    "path": path,
                    "body": body,
                    "timestamp": timestamp,
                });

                let mut request = self.http_client.post(remote.url.clone()).json(&payload);
                if let Some(token) = &remote.token {
                    request = request.bearer_auth(token);
                }

                let response = request.send().await?;
                let status = response.status();
                if !status.is_success() {
                    let body = response.text().await.unwrap_or_default();
                    return Err(Error::Api { status, body });
                }

                Ok(response.json().await?)
            }
            BuilderType::Unavailable => Err(Error::BuilderCredentialsUnavailable),
        }
    }
}

fn has_valid_local_creds(creds: &BuilderApiKeyCreds) -> bool {
    !creds.key.trim().is_empty()
        && !creds.secret.expose_secret().trim().is_empty()
        && !creds.passphrase.expose_secret().trim().is_empty()
}

fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is before unix epoch")
        .as_secs()
}

fn build_hmac_signature(
    secret: &str,
    timestamp: u64,
    method: &str,
    request_path: &str,
    body: Option<&str>,
) -> Result<String> {
    let decoded_secret = decode_secret(secret)?;
    let mut mac = Hmac::<Sha256>::new_from_slice(&decoded_secret)?;

    let mut message = format!("{timestamp}{method}{request_path}");
    if let Some(body) = body {
        message.push_str(body);
    }

    mac.update(message.as_bytes());
    Ok(URL_SAFE.encode(mac.finalize().into_bytes()))
}

fn decode_secret(secret: &str) -> Result<Vec<u8>> {
    let trimmed = secret.trim();
    if let Ok(bytes) = URL_SAFE.decode(trimmed) {
        return Ok(bytes);
    }
    if let Ok(bytes) = STANDARD.decode(trimmed) {
        return Ok(bytes);
    }

    let mut padded = trimmed.to_owned();
    let remainder = padded.len() % 4;
    if remainder != 0 {
        padded.push_str(&"=".repeat(4 - remainder));
    }

    URL_SAFE
        .decode(&padded)
        .or_else(|_| STANDARD.decode(&padded))
        .map_err(Into::into)
}
