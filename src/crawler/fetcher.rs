use crate::crawler::http_client::HttpClient;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSpec {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResponse {
    pub url: String,
    pub status: u16,
    pub body: String,
    pub content_type: Option<String>,
}

pub async fn fetch(client: &HttpClient, req: RequestSpec) -> anyhow::Result<FetchResponse> {
    const MAX_RETRIES: usize = 2;
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 0..=MAX_RETRIES {
        let req = req.clone();
        let mut builder = match req.method {
            HttpMethod::GET => client.client().get(&req.url),
            HttpMethod::POST => client.client().post(&req.url),
        };

        let mut has_content_type = false;
        for (k, v) in &req.headers {
            if k.to_lowercase() == "content-type" {
                has_content_type = true;
            }
            builder = builder.header(k, v);
        }

        if let Some(body) = req.body {
            if matches!(req.method, HttpMethod::POST) && !has_content_type {
                builder = builder.header(
                    reqwest::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                );
            }
            println!("DEBUG: fetch sending body: {}", body);
            builder = builder.body(body);
        }

        println!(
            "DEBUG: fetch executing {} request to: {}",
            match req.method {
                HttpMethod::GET => "GET",
                HttpMethod::POST => "POST",
            },
            req.url
        );
        match builder.send().await {
            Ok(res) => {
                let status = res.status().as_u16();
                println!("DEBUG: fetch response status: {}", status);
                let url = res.url().to_string();
                let content_type = res
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());
                let body = res.text().await?;
                if status >= 500 && attempt < MAX_RETRIES {
                    last_err = Some(anyhow::anyhow!("server error status {}", status));
                } else {
                    return Ok(FetchResponse {
                        url,
                        status,
                        body,
                        content_type,
                    });
                }
            }
            Err(e) => {
                last_err = Some(e.into());
            }
        }

        if attempt < MAX_RETRIES {
            let backoff = 200u64 * (attempt as u64 + 1);
            sleep(Duration::from_millis(backoff)).await;
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("fetch failed")))
}
