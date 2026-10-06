use crate::api::auth::AuthContext;
use crate::api::AppState;
use crate::error::error::{ApiResponse, AppError};
use crate::model::book_group::BookGroup;
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct GroupIdParam {
    #[serde(rename = "groupId")]
    group_id: Option<i64>,
}

pub async fn get_book_groups(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state
        .user_service
        .resolve_read_user_ns(auth.access_token(), auth.secure_key(), auth.user_ns())
        .await
        .map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let groups = state.book_group_service.get_groups(&user_ns).await?;
    Ok(Json(ApiResponse::ok(
        serde_json::to_value(groups).unwrap_or_default(),
    )))
}

pub async fn save_book_group(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(group): Json<BookGroup>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state
        .user_service
        .resolve_write_user_ns(auth.access_token(), auth.secure_key(), auth.user_ns())
        .await
        .map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    state.book_group_service.save_group(&user_ns, group).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!("success"))))
}

pub async fn delete_book_group(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(param): Json<GroupIdParam>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state
        .user_service
        .resolve_write_user_ns(auth.access_token(), auth.secure_key(), auth.user_ns())
        .await
        .map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let gid = param
        .group_id
        .ok_or_else(|| AppError::BadRequest("groupId required".to_string()))?;
    state.book_group_service.delete_group(&user_ns, gid).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!("success"))))
}

pub async fn save_book_group_order(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(groups): Json<Vec<BookGroup>>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state
        .user_service
        .resolve_write_user_ns(auth.access_token(), auth.secure_key(), auth.user_ns())
        .await
        .map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    state
        .book_group_service
        .save_groups(&user_ns, &groups)
        .await?;
    Ok(Json(ApiResponse::ok(serde_json::json!("success"))))
}

#[derive(Debug, Deserialize)]
pub struct SaveBookGroupIdParam {
    #[serde(rename = "bookUrl")]
    book_url: Option<String>,
    #[serde(rename = "groupId")]
    group_id: Option<i64>,
}

pub async fn save_book_group_id(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(param): Json<SaveBookGroupIdParam>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state
        .user_service
        .resolve_write_user_ns(auth.access_token(), auth.secure_key(), auth.user_ns())
        .await
        .map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let url = param
        .book_url
        .ok_or_else(|| AppError::BadRequest("bookUrl required".to_string()))?;
    let gid = param.group_id.unwrap_or(0);
    let mut book = state
        .book_service
        .get_shelf_book(&user_ns, &url)
        .await?
        .ok_or_else(|| AppError::NotFound("Book not found".to_string()))?;
    book.group = Some(gid);
    state.book_service.save_book(&user_ns, book).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!("success"))))
}

#[derive(Debug, Deserialize)]
pub struct MultiBookGroupParam {
    #[serde(rename = "bookUrls")]
    book_urls: Option<Vec<String>>,
    #[serde(rename = "groupId")]
    group_id: Option<i64>,
}

pub async fn add_book_group_multi(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(param): Json<MultiBookGroupParam>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state
        .user_service
        .resolve_write_user_ns(auth.access_token(), auth.secure_key(), auth.user_ns())
        .await
        .map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let urls = param.book_urls.unwrap_or_default();
    let gid = param.group_id.unwrap_or(0);
    // 纯单一分组模式：直接将书籍 group 设置为指定 groupId，不进行按位或
    for url in urls {
        if let Some(mut book) = state.book_service.get_shelf_book(&user_ns, &url).await? {
            book.group = Some(gid);
            let _ = state.book_service.save_book(&user_ns, book).await;
        }
    }
    Ok(Json(ApiResponse::ok(serde_json::json!("success"))))
}

pub async fn remove_book_group_multi(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(param): Json<MultiBookGroupParam>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state
        .user_service
        .resolve_write_user_ns(auth.access_token(), auth.secure_key(), auth.user_ns())
        .await
        .map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let urls = param.book_urls.unwrap_or_default();
    let gid = param.group_id.unwrap_or(0);
    // 纯单一分组模式：如果书籍正好属于该组，移出即重置为 0 (未分组)
    for url in urls {
        if let Some(mut book) = state.book_service.get_shelf_book(&user_ns, &url).await? {
            if book.group == Some(gid) {
                book.group = Some(0);
                let _ = state.book_service.save_book(&user_ns, book).await;
            }
        }
    }
    Ok(Json(ApiResponse::ok(serde_json::json!("success"))))
}
