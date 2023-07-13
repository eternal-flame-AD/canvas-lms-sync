use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{header::HeaderValue, IntoUrl};
use serde::{de::DeserializeOwned, Deserialize};

pub mod files;
pub mod modules;

pub struct Client {
    reqwest: reqwest::Client,
    host: String,
    auth_bearer: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub errors: Vec<ApiErrorDetail>,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorDetail {
    pub message: String,
}

pub struct ResponsePagination {
    current: String,
    prev: Option<String>,
    next: Option<String>,
    first: String,
    last: String,
}

impl From<&HeaderValue> for ResponsePagination {
    fn from(value: &HeaderValue) -> Self {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"<(.+?)>; rel="(.+?)""#).unwrap());

        RE.captures_iter(value.to_str().unwrap())
            .map(|cap| (cap[2].to_string(), cap[1].to_string()))
            .fold(
                ResponsePagination {
                    current: String::new(),
                    prev: None,
                    next: None,
                    first: String::new(),
                    last: String::new(),
                },
                |mut acc, (rel, url)| {
                    match rel.as_str() {
                        "current" => acc.current = url,
                        "prev" => acc.prev = Some(url),
                        "next" => acc.next = Some(url),
                        "first" => acc.first = url,
                        "last" => acc.last = url,
                        _ => (),
                    }
                    acc
                },
            )
    }
}

#[derive(Debug)]
pub enum Error {
    ApiError(ApiError),
    ReqwestError(reqwest::Error),
}

pub type ApiResult<T> = Result<T, Error>;

impl Client {
    pub fn new(host: String, auth_bearer: String) -> Self {
        Self {
            reqwest: reqwest::Client::new(),
            host,
            auth_bearer,
        }
    }
    pub fn build_url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.host.strip_suffix("/").unwrap(),
            path.strip_prefix("/").unwrap()
        )
    }
    pub async fn make_json_request<T: DeserializeOwned, U: IntoUrl>(
        &self,
        url: U,
    ) -> ApiResult<(T, Option<ResponsePagination>)> {
        let response = self
            .reqwest
            .get(url)
            .bearer_auth(self.auth_bearer.clone())
            .send()
            .await
            .map_err(Error::ReqwestError)?;

        if response.status() != 200 {
            let json = response.json().await.map_err(Error::ReqwestError)?;
            return Err(Error::ApiError(json));
        }
        let pagination = response.headers().get("Link").map(|v| v.into());
        let json = response.json().await.map_err(Error::ReqwestError)?;
        Ok((json, pagination))
    }
}
