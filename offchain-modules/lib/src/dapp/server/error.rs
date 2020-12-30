use actix_web::dev::HttpResponseBuilder;
use actix_web::http::{header, StatusCode};
use actix_web::{error, HttpResponse};
use crossbeam_channel::{RecvTimeoutError, SendError};
use derive_more::Display;

// TODO: split user params error and server error
#[derive(Debug, Display)]
pub enum RpcError {
    #[display(fmt = "bad request data: {}", _0)]
    BadRequest(String),
}

impl From<anyhow::Error> for RpcError {
    fn from(e: anyhow::Error) -> Self {
        Self::BadRequest(e.to_string())
    }
}

impl From<SendError<String>> for RpcError {
    fn from(e: SendError<String>) -> Self {
        Self::BadRequest(e.to_string())
    }
}

impl From<RecvTimeoutError> for RpcError {
    fn from(e: RecvTimeoutError) -> Self {
        Self::BadRequest(e.to_string())
    }
}

impl From<&str> for RpcError {
    fn from(e: &str) -> Self {
        Self::BadRequest(e.to_string())
    }
}

impl From<String> for RpcError {
    fn from(e: String) -> Self {
        Self::BadRequest(e)
    }
}

impl error::ResponseError for RpcError {
    fn error_response(&self) -> HttpResponse {
        let error_string = self.to_string();
        log::error!("api return error: {}", error_string);
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(error_string)
    }

    fn status_code(&self) -> StatusCode {
        match &*self {
            Self::BadRequest(_e) => StatusCode::BAD_REQUEST,
        }
    }
}
