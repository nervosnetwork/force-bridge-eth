use super::ReplayResistTask;
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::{header, StatusCode};
use actix_web::{error, HttpResponse};
use derive_more::Display;
use tokio::sync::mpsc::error::TrySendError;

#[derive(Debug, Display)]
pub enum RpcError {
    #[display(fmt = "bad request data: {}", _0)]
    BadRequest(String),
    #[display(fmt = "too many request: {}", _0)]
    TooManyRequest(String),
    #[display(fmt = "server error: {}", _0)]
    ServerError(String),
}

impl From<TrySendError<ReplayResistTask>> for RpcError {
    fn from(e: TrySendError<ReplayResistTask>) -> Self {
        Self::TooManyRequest(e.to_string())
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
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::TooManyRequest(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::ServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
