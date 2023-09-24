#[cfg(not(target_arch = "wasm32"))]
mod native_client;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use native_client::stream_request;

#[cfg(target_arch = "wasm32")]
mod wasm_client;

use log::{error, trace, warn};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::from_str;

use crate::{Error, MeilisearchCommunicationError, MeilisearchError};

pub(crate) use method::Method;
mod method {
    #[derive(Debug)]
    pub enum Method<Q, B> {
        Get { query: Q },
        Post { query: Q, body: B },
        Patch { query: Q, body: B },
        Put { query: Q, body: B },
        Delete { query: Q },
    }

    impl<Q, B> Method<Q, B> {
        pub fn query(&self) -> &Q {
            match self {
                Method::Get { query } => query,
                Method::Post { query, .. } => query,
                Method::Patch { query, .. } => query,
                Method::Put { query, .. } => query,
                Method::Delete { query } => query,
            }
        }

        pub fn http_method(&self) -> http::Method {
            match self {
                Method::Get { .. } => http::Method::GET,
                Method::Post { .. } => http::Method::POST,
                Method::Patch { .. } => http::Method::PATCH,
                Method::Put { .. } => http::Method::PUT,
                Method::Delete { .. } => http::Method::DELETE,
            }
        }
    }
}

pub(crate) async fn request<Q, B, O>(
    url: &str,
    apikey: Option<&str>,
    method: Method<Q, B>,
    expected_status_code: u16,
) -> Result<O, Error>
where
    Q: Serialize,
    B: Serialize,
    O: DeserializeOwned + 'static,
{
    const CONTENT_TYPE: &str = "application/json";

    #[cfg(not(target_arch = "wasm32"))]
    use self::native_client::{NativeRequestClient, SerializeBodyTransform};
    #[cfg(not(target_arch = "wasm32"))]
    return NativeRequestClient::<SerializeBodyTransform, _>::request(
        url,
        apikey,
        method,
        CONTENT_TYPE,
        expected_status_code,
    )
    .await;

    #[cfg(target_arch = "wasm32")]
    return self::wasm_client::BrowserRequestClient::request(
        url,
        apikey,
        method,
        CONTENT_TYPE,
        expected_status_code,
    )
    .await;
}

trait RequestClient<B0>: Sized {
    type Request;
    type Response;

    fn new(url: String) -> Self;

    fn with_authorization_header(self, bearer_token_value: &str) -> Self;

    fn with_user_agent_header(self, user_agent_value: &str) -> Self;

    fn with_method(self, http_method: http::Method) -> Self;

    fn add_body<Q>(self, method: Method<Q, B0>, content_type: &str) -> Self::Request;

    async fn send_request(request: Self::Request) -> Result<Self::Response, Error>;

    fn extract_status_code(response: &Self::Response) -> u16;

    async fn response_to_text(response: Self::Response) -> Result<String, Error>;

    async fn request<T, Q>(
        url: &str,
        apikey: Option<&str>,
        method: Method<Q, B0>,
        content_type: &str,
        expected_status_code: u16,
    ) -> Result<T, Error>
    where
        Q: Serialize,
        T: DeserializeOwned + 'static,
    {
        let mut request_client = Self::new(add_query_parameters(url, method.query())?)
            .with_method(method.http_method())
            .with_user_agent_header(&qualified_version());

        if let Some(apikey) = apikey {
            request_client = request_client.with_authorization_header(&format!("Bearer {apikey}"));
        }

        let response = Self::send_request(request_client.add_body(method, content_type)).await?;
        let status = Self::extract_status_code(&response);
        let text = Self::response_to_text(response).await?;

        Self::parse_response(status, expected_status_code, &text, url.to_string())
    }

    fn parse_response<Output: DeserializeOwned>(
        status_code: u16,
        expected_status_code: u16,
        mut body: &str,
        url: String,
    ) -> Result<Output, Error> {
        if body.is_empty() {
            body = "null"
        }

        if status_code == expected_status_code {
            match from_str::<Output>(body) {
                Ok(output) => {
                    trace!("Request succeed");
                    return Ok(output);
                }
                Err(e) => {
                    error!("Request succeeded but failed to parse response");
                    return Err(Error::ParseError(e));
                }
            };
        }

        warn!(
            "Expected response code {}, got {}",
            expected_status_code, status_code
        );

        match from_str::<MeilisearchError>(body) {
            Ok(e) => Err(Error::from(e)),
            Err(e) => {
                if status_code >= 400 {
                    return Err(Error::MeilisearchCommunication(
                        MeilisearchCommunicationError {
                            status_code,
                            message: None,
                            url,
                        },
                    ));
                }
                Err(Error::ParseError(e))
            }
        }
    }
}

pub fn qualified_version() -> String {
    const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

    format!("Meilisearch Rust (v{})", VERSION.unwrap_or("unknown"))
}

pub fn add_query_parameters<Query: Serialize>(url: &str, query: &Query) -> Result<String, Error> {
    let query = yaup::to_string(query)?;

    Ok(if query.is_empty() {
        url.into()
    } else {
        format!("{url}?{query}")
    })
}
