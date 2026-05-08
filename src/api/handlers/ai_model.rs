use axum::{extract::State, Json};
use serde_json::Value;

use crate::api::{auth::AuthContext, AppState};
use crate::error::error::{ApiResponse, AppError};
use crate::model::ai_model::AiModelConfig;

pub async fn get_ai_model_config(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let is_admin = is_ai_model_admin(&state, &auth).await?;
    let can_use_server_model = state
        .user_service
        .can_use_ai_model(auth.access_token(), auth.secure_key())
        .await?;
    let config = state.ai_model_service.get().await?;
    let visible_config = if is_admin {
        config
    } else {
        config.without_secrets()
    };
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "config": visible_config,
        "canUseServerModel": can_use_server_model,
        "isAdmin": is_admin,
    }))))
}

pub async fn save_ai_model_config(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(config): Json<AiModelConfig>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    if !is_ai_model_admin(&state, &auth).await? {
        return Ok(Json(ApiResponse::err_with_data(
            "请输入管理密码",
            Value::String("NEED_SECURE_KEY".to_string()),
        )));
    }
    let saved = state.ai_model_service.save(config).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "config": saved,
        "canUseServerModel": true,
        "isAdmin": true,
    }))))
}

async fn is_ai_model_admin(state: &AppState, auth: &AuthContext) -> Result<bool, AppError> {
    if !state.user_service.secure_enabled() {
        return Ok(true);
    }
    state
        .user_service
        .is_admin(auth.access_token(), auth.secure_key())
        .await
}
