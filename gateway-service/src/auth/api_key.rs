use axum::http::Request;
use crate::auth::error::AuthError;
use crate::auth::jwt::UserInfo;
use crate::config::CONFIG;
use chrono::{DateTime, Utc};

/// 通过API Key进行认证
pub async fn authenticate_api_key<B>(request: &Request<B>) -> Result<UserInfo, AuthError> {
    let config = CONFIG.read().await;
    let api_key_config = &config.auth.api_key;
    
    // 从请求头中提取API Key
    let api_key = request.headers()
        .get(&api_key_config.header_name)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
        .ok_or(AuthError::InvalidApiKey)?;
    
    // 查找API Key
    let api_key_info = api_key_config.api_keys.get(&api_key)
        .ok_or(AuthError::InvalidApiKey)?;
    
    // 检查API Key是否启用
    if !api_key_info.enabled {
        return Err(AuthError::InvalidApiKey);
    }
    
    // 检查API Key是否过期
    if let Some(expires_at) = &api_key_info.expires_at {
        match DateTime::parse_from_rfc3339(expires_at) {
            Ok(expiry_time) => {
                if expiry_time < Utc::now() {
                    return Err(AuthError::ApiKeyExpired);
                }
            },
            Err(_) => {
                return Err(AuthError::InternalError("无效的API Key过期时间格式".to_string()));
            }
        }
    }
    
    // 获取用户ID
    let user_id = api_key_info.user_id
        .ok_or(AuthError::InternalError("API Key未关联用户ID".to_string()))?;
    
    // 构建用户信息
    let user_info = UserInfo {
        user_id,
        username: api_key_info.name.clone(),
        roles: api_key_info.permissions.clone(),
        extra: Default::default(),
    };
    
    Ok(user_info)
} 