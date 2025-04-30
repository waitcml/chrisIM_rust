use thiserror::Error;
use axum::{
    response::{Response, IntoResponse},
    http::StatusCode,
    Json,
};
use serde_json::json;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("未授权访问")]
    Unauthorized,
    
    #[error("Token已过期")]
    TokenExpired,
    
    #[error("Token无效")]
    InvalidToken,
    
    #[error("签发者无效")]
    InvalidIssuer,
    
    #[error("没有足够的权限")]
    InsufficientPermissions,
    
    #[error("API Key无效")]
    InvalidApiKey,
    
    #[error("API Key已过期")]
    ApiKeyExpired,
    
    #[error("OAuth2认证失败: {0}")]
    OAuth2Error(String),
    
    #[error("内部认证错误: {0}")]
    InternalError(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "未授权访问".to_string()),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token已过期".to_string()),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Token无效".to_string()),
            AuthError::InvalidIssuer => (StatusCode::UNAUTHORIZED, "签发者无效".to_string()),
            AuthError::InsufficientPermissions => (StatusCode::FORBIDDEN, "没有足够的权限".to_string()),
            AuthError::InvalidApiKey => (StatusCode::UNAUTHORIZED, "API Key无效".to_string()),
            AuthError::ApiKeyExpired => (StatusCode::UNAUTHORIZED, "API Key已过期".to_string()),
            AuthError::OAuth2Error(msg) => (StatusCode::UNAUTHORIZED, msg),
            AuthError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "内部认证错误".to_string()),
        };
        
        let json = Json(json!({
            "error": status.as_u16(),
            "message": message,
        }));
        
        (status, json).into_response()
    }
} 