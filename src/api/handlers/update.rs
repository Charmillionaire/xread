use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::Value;

use crate::api::auth::AuthContext;
use crate::api::AppState;
use crate::error::error::{ApiResponse, AppError};

#[derive(Debug, Deserialize)]
pub struct VersionUpdateQuery {
    pub force: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DismissVersionUpdateRequest {
    pub version: Option<String>,
}

pub async fn get_version_update(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(query): Query<VersionUpdateQuery>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    if !can_manage_updates(&state, &auth).await? {
        return Ok(Json(ApiResponse::err_with_data(
            "请输入管理密码",
            Value::String("NEED_SECURE_KEY".to_string()),
        )));
    }
    let info = state
        .update_service
        .check(query.force.unwrap_or(false))
        .await?;
    Ok(Json(ApiResponse::ok(
        serde_json::to_value(info).map_err(|err| AppError::BadRequest(err.to_string()))?,
    )))
}

pub async fn dismiss_version_update(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<DismissVersionUpdateRequest>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    if !can_manage_updates(&state, &auth).await? {
        return Ok(Json(ApiResponse::err_with_data(
            "请输入管理密码",
            Value::String("NEED_SECURE_KEY".to_string()),
        )));
    }
    let version = req.version.unwrap_or_default();
    let info = state.update_service.dismiss(&version).await?;
    Ok(Json(ApiResponse::ok(
        serde_json::to_value(info).map_err(|err| AppError::BadRequest(err.to_string()))?,
    )))
}

async fn can_manage_updates(state: &AppState, auth: &AuthContext) -> Result<bool, AppError> {
    if !state.user_service.secure_enabled() {
        return Ok(true);
    }
    state
        .user_service
        .is_admin(auth.access_token(), auth.secure_key())
        .await
}
