use crate::models::HttpResponse;
use base64::Engine as _;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpVerb {
    Get,
    Post,
    Put,
    Delete,
    Head,
}

impl HttpVerb {
    pub fn from_input(method: Option<String>) -> Result<Self, String> {
        match normalized_http_method(method).as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "HEAD" => Ok(Self::Head),
            other => Err(format!("Unsupported HTTP method: {}", other)),
        }
    }

    pub fn build_request(self, client: &reqwest::Client, url: &str) -> reqwest::RequestBuilder {
        match self {
            Self::Get => client.get(url),
            Self::Post => client.post(url),
            Self::Put => client.put(url),
            Self::Delete => client.delete(url),
            Self::Head => client.head(url),
        }
    }
}

pub fn normalized_http_method(method: Option<String>) -> String {
    match method {
        Some(method) => method.to_uppercase(),
        None => "GET".to_string(),
    }
}

pub fn is_image_request(url: &str, content_type: &str) -> bool {
    let ct = content_type.to_lowercase();
    ct.starts_with("image/")
        || url.contains(".jpg")
        || url.contains(".jpeg")
        || url.contains(".png")
        || url.contains(".gif")
        || url.contains(".webp")
}

pub fn is_image_response(url: &str, headers: &HashMap<String, String>) -> bool {
    let content_type = headers.get("content-type").cloned().unwrap_or_default();
    is_image_request(url, &content_type)
}

pub fn lowercased_response_headers(
    headers: &reqwest::header::HeaderMap,
) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for (name, value) in headers {
        if let Ok(value_str) = value.to_str() {
            out.insert(name.to_string().to_lowercase(), value_str.to_string());
        }
    }
    out
}

pub fn map_image_body_result(read_result: Result<Vec<u8>, String>) -> Result<String, String> {
    read_result
        .map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes))
        .map_err(|error| format!("Failed to read response bytes: {}", error))
}

pub fn map_text_body_result(read_result: Result<String, String>) -> Result<String, String> {
    read_result.map_err(|error| format!("Failed to read response body: {}", error))
}

pub async fn response_body_as_text_or_base64(
    response: reqwest::Response,
    url: &str,
    response_headers: &HashMap<String, String>,
) -> Result<String, String> {
    if is_image_response(url, response_headers) {
        map_image_body_result(
            response
                .bytes()
                .await
                .map(|bytes| bytes.to_vec())
                .map_err(|error| error.to_string()),
        )
    } else {
        map_text_body_result(response.text().await.map_err(|error| error.to_string()))
    }
}

pub async fn perform_http_request(
    url: String,
    method: Option<String>,
    headers: Option<HashMap<String, String>>,
) -> Result<HttpResponse, String> {
    let client = reqwest::Client::new();
    let verb = HttpVerb::from_input(method)?;
    let mut request = verb.build_request(&client, &url);

    if let Some(headers_map) = headers {
        for (key, value) in headers_map {
            request = request.header(&key, &value);
        }
    }

    let response = match request.send().await {
        Ok(response) => response,
        Err(error) => return Err(format!("HTTP request failed: {}", error)),
    };

    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("")
        .to_string();

    let response_headers = lowercased_response_headers(response.headers());
    let body = response_body_as_text_or_base64(response, &url, &response_headers).await?;

    Ok(HttpResponse {
        status,
        status_text,
        body,
        headers: response_headers,
    })
}

#[cfg(test)]
#[path = "http_tests.rs"]
mod tests;
