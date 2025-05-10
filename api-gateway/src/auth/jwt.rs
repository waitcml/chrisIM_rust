use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Serialize, Deserialize};
use axum::http::Request;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::config::CONFIG;
use crate::auth::error::AuthError;

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// 用户ID
    pub user_id: i64,
    /// 用户名
    pub username: String,
    /// 用户角色
    pub roles: Vec<String>,
    /// 额外信息
    #[serde(default)]
    pub extra: std::collections::HashMap<String, String>,
}

/// JWT Token中的声明信息
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// 主题 (用户ID)
    pub sub: String,
    /// 签发者
    pub iss: Option<String>,
    /// 过期时间
    pub exp: u64,
    /// 签发时间
    pub iat: u64,
    /// 用户名
    pub username: String,
    /// 用户角色
    #[serde(default)]
    pub roles: Vec<String>,
    /// 额外信息
    #[serde(default)]
    pub extra: std::collections::HashMap<String, String>,
}

/// 从请求中验证JWT Token
pub async fn authenticate_jwt<B>(request: &Request<B>) -> Result<UserInfo, AuthError> {
    let config = CONFIG.read().await;
    let jwt_config = &config.auth.jwt;
    
    // 从请求头中提取token
    let token = extract_token(request, &jwt_config.header_name, &jwt_config.header_prefix)
        .ok_or(AuthError::Unauthorized)?;
    
    // 解码并验证token
    let mut validation = Validation::new(Algorithm::HS256);
    if jwt_config.verify_issuer && !jwt_config.allowed_issuers.is_empty() {
        validation.iss = Some(jwt_config.allowed_issuers.clone().into_iter().collect());
    }
    
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(jwt_config.secret.as_bytes()),
        &validation
    ).map_err(|e| {
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidIssuer => AuthError::InvalidIssuer,
            _ => AuthError::InvalidToken,
        }
    })?;
    
    // 检查token是否过期
    let now = SystemTime::now().duration_since(UNIX_EPOCH)
        .map_err(|e| AuthError::InternalError(e.to_string()))?
        .as_secs();
    
    if token_data.claims.exp <= now {
        return Err(AuthError::TokenExpired);
    }
    
    // 构建用户信息
    let user_info = UserInfo {
        user_id: token_data.claims.sub.parse::<i64>()
            .map_err(|_| AuthError::InvalidToken)?,
        username: token_data.claims.username,
        roles: token_data.claims.roles,
        extra: token_data.claims.extra,
    };
    
    Ok(user_info)
}

/// 从请求头中提取token
fn extract_token<B>(request: &Request<B>, header_name: &str, header_prefix: &str) -> Option<String> {
    request.headers()
        .get(header_name)
        .and_then(|value| value.to_str().ok())
        .and_then(|auth_header| {
            if auth_header.starts_with(header_prefix) {
                Some(auth_header[header_prefix.len()..].to_string())
            } else {
                None
            }
        })
}

/// 获取当前时间戳
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
} 