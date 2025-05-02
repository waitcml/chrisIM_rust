use aws_sdk_s3::error::SdkError;
use serde::de::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("内部服务错误: {0}")]
    Internal(String),

    #[error("认证失败: {0}")]
    Authentication(String),

    #[error("授权失败: {0}")]
    Authorization(String),

    #[error("资源不存在: {0}")]
    NotFound(String),

    #[error("请求无效: {0}")]
    BadRequest(String),

    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis错误: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("IO错误: {0}")]
    IO(#[from] std::io::Error),

    #[error("JSON错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("JWT错误: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("gRPC传输错误: {0}")]
    Tonic(#[from] tonic::transport::Error),

    #[error("gRPC状态错误: {0}")]
    TonicStatus(#[from] tonic::Status),

    OSSError,
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::Internal(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::Internal(err.to_string())
    }
}

// 添加UUID解析错误的From实现
impl From<uuid::Error> for Error {
    fn from(err: uuid::Error) -> Self {
        Error::BadRequest(format!("UUID解析错误: {}", err))
    }
}

// 从Error转换为tonic::Status，用于gRPC响应
impl From<Error> for tonic::Status {
    fn from(error: Error) -> Self {
        match error {
            Error::NotFound(msg) => tonic::Status::not_found(msg),
            Error::Authentication(msg) => tonic::Status::unauthenticated(msg),
            Error::Authorization(msg) => tonic::Status::permission_denied(msg),
            Error::BadRequest(msg) => tonic::Status::invalid_argument(msg),
            _ => tonic::Status::internal(error.to_string()),
        }
    }
}

impl<E> From<SdkError<E>> for Error
where
    E: StdError + 'static,
{
    fn from(sdk_error: SdkError<E>) -> Self {
        let kind = Error::OSSError;

        let details = sdk_error.to_string();

        Self::with_details(kind, details)
    }
}

// 从Error转换为axum::http::StatusCode，用于HTTP响应
impl From<Error> for axum::http::StatusCode {
    fn from(error: Error) -> Self {
        use axum::http::StatusCode;
        match error {
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::Authentication(_) => StatusCode::UNAUTHORIZED,
            Error::Authorization(_) => StatusCode::FORBIDDEN,
            Error::BadRequest(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
} 