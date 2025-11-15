use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub enum AppError {
    // 专门用于处理读取配置文件失败的错误
    ConfigReadError(String),
    ConfigWriteError(String),
    ConfigReloadError(String),
    ConfigResetByIndexError(String),
    BadBodyError(String),
    GameIsRunning,
    DataServerFucError(String),
    SetServerConfigXmlErrror(String),
    StopProcessError(String),
    GetS3ClientError(String),
    UnzipError(String),
    DownloadError(String),
    IOError(std::io::Error),
    KillCommandError(String)
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::ConfigReadError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read config file: {}", msg),
            ),
            AppError::ConfigWriteError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to write config file: {}", msg),
            ),
            AppError::ConfigReloadError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to reload config file: {}", msg),
            ),
            AppError::BadBodyError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Bed body: {}", msg),
            ),
            AppError::ConfigResetByIndexError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to reset config file by index: {}", msg),
            ),
            AppError::GameIsRunning => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("gamme is running"),
            ),
            AppError::DataServerFucError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("visit dataserver func error: {}", msg),
            ),
            AppError::SetServerConfigXmlErrror(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("set serverconfig.xml error: {}", msg),
            ),
            AppError::StopProcessError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("stop process error: {}", msg),
            ),
            AppError::GetS3ClientError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("get s3 client error: {}", msg),
            ),
            AppError::DownloadError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("download file error: {}", msg),
            ),
            AppError::IOError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("io error: {}", msg),
            ),
            AppError::UnzipError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unzip error: {}", msg),
            ),
            AppError::KillCommandError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("kill command error: {}", msg),
            ),
        };

        (status, error_message).into_response()
    }
}