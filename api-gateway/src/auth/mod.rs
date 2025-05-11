pub mod jwt;
pub mod api_key;
pub mod oauth2;
pub mod middleware;
pub mod error;

use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::body::{Bytes, Body};
use crate::config::CONFIG;
use crate::auth::error::AuthError;
use http_body_util::BodyExt;

/// 统一认证入口
pub async fn authenticate<B>(request: Request<B>, next: Next) -> Result<Response, AuthError>
where 
    B: axum::body::HttpBody<Data = Bytes> + Send + 'static,
    B::Error: std::fmt::Display + Send + Sync + 'static
{
    let config = CONFIG.read().await;
    
    // 检查路径是否在白名单中
    let path = request.uri().path();
    if config.auth.path_whitelist.iter().any(|p| path.starts_with(p)) {
        // 白名单路径，直接放行
        // 转换请求体类型
        let (parts, body) = request.into_parts();
        let bytes = body.collect().await
            .map_err(|_| AuthError::InternalError("无法读取请求体".to_string()))?
            .to_bytes();
        let new_body = Body::from(bytes);
        let new_request = Request::from_parts(parts, new_body);
        
        return Ok(next.run(new_request).await);
    }
    
    // 检查IP是否在白名单中
    if let Some(ip) = get_client_ip(&request) {
        if config.auth.ip_whitelist.contains(&ip) {
            // IP白名单，直接放行
            // 转换请求体类型
            let (parts, body) = request.into_parts();
            let bytes = body.collect().await
                .map_err(|_| AuthError::InternalError("无法读取请求体".to_string()))?
                .to_bytes();
            let new_body = Body::from(bytes);
            let new_request = Request::from_parts(parts, new_body);
            
            return Ok(next.run(new_request).await);
        }
    }
    
    // 尝试各种认证方式，先尝试从header获取用户信息
    let result = if config.auth.jwt.enabled {
        jwt::authenticate_jwt(&request).await
    } else if config.auth.api_key.enabled {
        api_key::authenticate_api_key(&request).await
    } else if config.auth.oauth2.enabled {
        oauth2::authenticate_oauth2(&request).await
    } else {
        Err(AuthError::Unauthorized)
    };
    
    match result {
        Ok(user_info) => {
            // 转换请求体类型
            let (parts, body) = request.into_parts();
            let bytes = body.collect().await
                .map_err(|_| AuthError::InternalError("无法读取请求体".to_string()))?
                .to_bytes();
            
            // 构建新的请求
            let mut new_parts = parts;
            
            // 将用户信息添加到扩展中
            new_parts.extensions.insert(user_info);
            
            let new_body = Body::from(bytes);
            let new_request = Request::from_parts(new_parts, new_body);
            
            // 继续处理请求
            Ok(next.run(new_request).await)
        },
        Err(e) => Err(e),
    }
}

/// 从请求中获取客户端IP
fn get_client_ip<B>(request: &Request<B>) -> Option<String> {
    request.headers()
        .get("X-Forwarded-For")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .or_else(|| {
            request.headers()
                .get("X-Real-IP")
                .and_then(|value| value.to_str().ok())
                .map(|s| s.to_string())
        })
} 