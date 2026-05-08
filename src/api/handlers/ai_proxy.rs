use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;

use crate::api::{auth::AuthContext, AppState};
use crate::error::error::{ApiResponse, AppError};
use crate::model::ai_model::{AiModelKind, ResolvedAiModelEndpoint};
use crate::model::ai_proxy::{
    ai_proxy_timeout, build_ai_proxy_url, format_ai_proxy_upstream_error,
    validate_ai_proxy_image_url, AiProxyImageRequest, AiProxyRequest,
};

const MAX_PROXY_IMAGE_BYTES: u64 = 20 * 1024 * 1024;

pub async fn ai_proxy(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<AiProxyRequest>,
) -> Result<Response, AppError> {
    require_proxy_user(&state, &auth).await?;
    let (endpoint, kind, path, mut body) = resolve_ai_proxy_target(&state, &auth, req).await?;
    if let Some(kind) = kind {
        apply_server_model_body_defaults(&endpoint, kind, &mut body);
    }
    let target = build_ai_proxy_url(&endpoint.base_url, &path, endpoint.use_full_url)
        .map_err(AppError::BadRequest)?;
    let client = ai_proxy_client()?;
    let mut builder = client
        .post(target)
        .header(reqwest::header::ACCEPT, "application/json")
        .json(&body);

    if let Some(api_key) = Some(endpoint.api_key.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        builder = builder.bearer_auth(api_key);
    }

    let upstream = builder.send().await.map_err(map_ai_proxy_http_error)?;
    response_from_upstream(upstream).await
}

async fn resolve_ai_proxy_target(
    state: &AppState,
    auth: &AuthContext,
    req: AiProxyRequest,
) -> Result<(ResolvedAiModelEndpoint, Option<AiModelKind>, String, Value), AppError> {
    if req.use_server_config {
        let can_use = state
            .user_service
            .can_use_ai_model(auth.access_token(), auth.secure_key())
            .await?;
        if !can_use {
            return Err(AppError::BadRequest(
                "当前账号没有使用后端模型配置的权限".to_string(),
            ));
        }
        let kind = req.kind.unwrap_or_else(|| infer_ai_model_kind(&req.path));
        let config = state.ai_model_service.get().await?;
        let endpoint = config.resolve(kind);
        if !endpoint.enabled
            || endpoint.base_url.trim().is_empty()
            || endpoint.model.trim().is_empty()
        {
            return Err(AppError::BadRequest(
                "后端模型配置未启用或不完整".to_string(),
            ));
        }
        return Ok((
            endpoint,
            Some(kind),
            default_ai_model_path(kind).to_string(),
            req.body,
        ));
    }

    Ok((
        ResolvedAiModelEndpoint {
            enabled: true,
            base_url: req.base_url,
            api_key: req.api_key.unwrap_or_default(),
            model: String::new(),
            use_full_url: req.full_url,
            image_size: None,
            voice: None,
            response_format: None,
        },
        None,
        req.path,
        req.body,
    ))
}

fn infer_ai_model_kind(path: &str) -> AiModelKind {
    match path {
        "/v1/images/generations" => AiModelKind::Image,
        "/v1/audio/speech" => AiModelKind::Speech,
        _ => AiModelKind::Text,
    }
}

fn default_ai_model_path(kind: AiModelKind) -> &'static str {
    match kind {
        AiModelKind::Text => "/v1/chat/completions",
        AiModelKind::Image => "/v1/images/generations",
        AiModelKind::Speech => "/v1/audio/speech",
    }
}

fn apply_server_model_body_defaults(
    endpoint: &ResolvedAiModelEndpoint,
    kind: AiModelKind,
    body: &mut Value,
) {
    if endpoint.model.is_empty() {
        return;
    }
    let Some(obj) = body.as_object_mut() else {
        return;
    };
    obj.insert("model".to_string(), Value::String(endpoint.model.clone()));
    if kind == AiModelKind::Image {
        if let Some(size) = endpoint
            .image_size
            .as_ref()
            .filter(|v| !v.trim().is_empty())
        {
            obj.insert("size".to_string(), Value::String(size.clone()));
        }
    }
    if kind == AiModelKind::Speech {
        if let Some(voice) = endpoint.voice.as_ref().filter(|v| !v.trim().is_empty()) {
            obj.insert("voice".to_string(), Value::String(voice.clone()));
        }
        if let Some(format) = endpoint
            .response_format
            .as_ref()
            .filter(|v| !v.trim().is_empty())
        {
            obj.insert("response_format".to_string(), Value::String(format.clone()));
        }
    }
}

pub async fn ai_proxy_image(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<AiProxyImageRequest>,
) -> Result<Response, AppError> {
    require_proxy_user(&state, &auth).await?;
    let target = validate_ai_proxy_image_url(&req.url).map_err(AppError::BadRequest)?;
    let client = ai_proxy_client()?;
    let upstream = client
        .get(target)
        .header(reqwest::header::ACCEPT, "image/*,*/*;q=0.8")
        .send()
        .await
        .map_err(map_ai_proxy_http_error)?;

    if let Some(length) = upstream.content_length() {
        if length > MAX_PROXY_IMAGE_BYTES {
            return Err(AppError::BadRequest("图片超过代理大小限制".to_string()));
        }
    }

    let status = upstream.status();
    let content_type = upstream
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| HeaderValue::from_str(v).ok());
    let body = upstream.bytes().await?;
    if body.len() as u64 > MAX_PROXY_IMAGE_BYTES {
        return Err(AppError::BadRequest("图片超过代理大小限制".to_string()));
    }
    if !status.is_success() {
        return Ok(build_upstream_error_response(status, &body));
    }
    Ok(build_response(status, content_type, body))
}

async fn require_proxy_user(state: &AppState, auth: &AuthContext) -> Result<(), AppError> {
    state
        .user_service
        .resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns())
        .await
        .map(|_| ())
        .map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))
}

async fn response_from_upstream(upstream: reqwest::Response) -> Result<Response, AppError> {
    let status = upstream.status();
    let content_type = upstream
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| HeaderValue::from_str(v).ok());
    let body = upstream.bytes().await?;
    if !status.is_success() {
        return Ok(build_upstream_error_response(status, &body));
    }
    Ok(build_response(status, content_type, body))
}

fn build_response(
    status: reqwest::StatusCode,
    content_type: Option<HeaderValue>,
    body: bytes::Bytes,
) -> Response {
    let status = StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let mut response = (status, body).into_response();
    if let Some(content_type) = content_type {
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, content_type);
    }
    response
}

fn build_upstream_error_response(status: reqwest::StatusCode, body: &bytes::Bytes) -> Response {
    let body_text = std::str::from_utf8(body).unwrap_or("");
    let message = format_ai_proxy_upstream_error(status.as_u16(), body_text);
    let status = StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let mut response = Json(ApiResponse::<Value>::err(message)).into_response();
    *response.status_mut() = status;
    response
}

fn ai_proxy_client() -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .timeout(ai_proxy_timeout())
        .build()
        .map_err(AppError::Http)
}

fn map_ai_proxy_http_error(error: reqwest::Error) -> AppError {
    if error.is_timeout() {
        return AppError::BadRequest("模型服务请求超时，请检查模型地址或稍后重试".to_string());
    }
    AppError::Http(error)
}
