use axum::http::Request;
use crate::auth::error::AuthError;
use crate::auth::jwt::UserInfo;
use crate::config::CONFIG;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;

/// OAuth2 token信息
#[derive(Debug, Serialize, Deserialize)]
struct TokenInfo {
    access_token: String,
    expires_in: Option<i64>,
    refresh_token: Option<String>,
    token_type: String,
}

/// 用户信息响应
#[derive(Debug, Serialize, Deserialize)]
struct UserInfoResponse {
    id: String,
    name: Option<String>,
    email: Option<String>,
    roles: Option<Vec<String>>,
}

/// 通过OAuth2认证
pub async fn authenticate_oauth2<B>(request: &Request<B>) -> Result<UserInfo, AuthError> {
    let config = CONFIG.read().await;
    let oauth_config = &config.auth.oauth2;
    
    // 从请求头中提取access_token
    let token = extract_oauth_token(request)
        .ok_or(AuthError::Unauthorized)?;
    
    // 验证token并获取用户信息
    let client = Client::new();
    
    // 这里简化了流程，实际上应该根据OAuth2提供商的API来获取用户信息
    // 通常会调用userinfo端点或通过introspection端点验证token
    let user_info_url = format!("{}/userinfo", oauth_config.token_url);
    
    // 发送请求获取用户信息
    let response = client.get(user_info_url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| AuthError::OAuth2Error(format!("获取用户信息失败: {}", e)))?;
    
    // 检查响应状态
    if !response.status().is_success() {
        return Err(AuthError::OAuth2Error("无效的OAuth2 token".to_string()));
    }
    
    // 解析用户信息
    let user_info_resp: Value = response.json()
        .await
        .map_err(|e| AuthError::OAuth2Error(format!("解析用户信息失败: {}", e)))?;
    
    // 提取用户ID
    let user_id = user_info_resp.get("sub")
        .or_else(|| user_info_resp.get("id"))
        .and_then(|v| v.as_str())
        .ok_or(AuthError::OAuth2Error("无法获取用户ID".to_string()))?
        .parse::<i64>()
        .map_err(|_| AuthError::OAuth2Error("无效的用户ID格式".to_string()))?;
    
    // 提取用户名
    let username = user_info_resp.get("name")
        .or_else(|| user_info_resp.get("username"))
        .or_else(|| user_info_resp.get("email"))
        .and_then(|v| v.as_str())
        .unwrap_or("oauth_user")
        .to_string();
    
    // 提取角色
    let roles = user_info_resp.get("roles")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_else(Vec::new);
    
    // 构建扩展信息
    let mut extra = HashMap::new();
    if let Some(email) = user_info_resp.get("email").and_then(|v| v.as_str()) {
        extra.insert("email".to_string(), email.to_string());
    }
    
    // 构建用户信息
    let user_info = UserInfo {
        user_id,
        username,
        roles,
        extra,
    };
    
    Ok(user_info)
}

/// 从请求中提取OAuth2 token
fn extract_oauth_token<B>(request: &Request<B>) -> Option<String> {
    // 首先尝试从Authorization头中提取
    let auth_header = request.headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok());
    
    if let Some(auth_header) = auth_header {
        if auth_header.starts_with("Bearer ") {
            return Some(auth_header[7..].to_string());
        }
    }
    
    // 然后尝试从查询参数中提取
    request.uri()
        .query()
        .and_then(|query| {
            query.split('&')
                .find(|pair| pair.starts_with("access_token="))
                .map(|pair| pair[13..].to_string())
        })
} 