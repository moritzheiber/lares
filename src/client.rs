use thiserror::Error;
use url::{ParseError, Url};

use crate::error::Result;

const MAXIMUM_REDIRECTION: u8 = 5;

/// User-Agent string (`LaresBot/<version> (+https://github.com/fanzeyi/lares)`)
const USER_AGENT: &str = concat!(
    "LaresBot/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/fanzeyi/lares)"
);

pub struct HttpClient;

#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error("Too many redirections")]
    TooManyRedirections,

    #[error("Missing location header for redirection response")]
    MissingLocationHeader,

    #[error("Unexpected status code: {}", _0)]
    UnexpectedStatusCode(tide::StatusCode),

    #[error("Internal Surf error")]
    SurfError(surf::Error),
}

impl HttpClient {
    pub async fn get(url: &str) -> Result<Vec<u8>> {
        let mut url = Url::parse(url)?;
        let mut redirection_count = 0;

        loop {
            let mut response = match surf::get(&url)
                .header("User-Agent", USER_AGENT)
                .header("Content-Length", "0")
                .await
            {
                Ok(resp) => resp,
                Err(e) => return Err(HttpClientError::SurfError(e).into()),
            };
            let status = response.status();

            if status.is_success() {
                break Ok(response.body_bytes().await?);
            }

            if status.is_redirection() {
                if redirection_count > MAXIMUM_REDIRECTION {
                    break Err(HttpClientError::TooManyRedirections.into());
                }

                redirection_count += 1;

                if let Some(location) = response.header("Location") {
                    match Url::parse(location.as_str()) {
                        Ok(parsed) => url = parsed,
                        Err(e) if e == ParseError::RelativeUrlWithoutBase => {
                            url.set_path(location.as_str());
                        }
                        Err(e) => break Err(e.into()),
                    }
                } else {
                    break Err(HttpClientError::MissingLocationHeader.into());
                }
            } else {
                break Err(HttpClientError::UnexpectedStatusCode(status).into());
            }
        }
    }
}
