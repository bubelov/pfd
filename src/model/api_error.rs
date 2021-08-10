use anyhow::Error;
use rocket::{
    async_trait,
    http::{ContentType, Status},
    request::Request,
    response::{self, Responder, Response},
    serde::Serialize,
};
use std::io::Cursor;
use tracing::error;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ApiError {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing)]
    pub error: Option<Error>,
}

impl ApiError {
    pub fn new(code: u16, error: Error) -> ApiError {
        ApiError {
            code: code,
            message: Status::from_code(code)
                .unwrap()
                .reason()
                .unwrap_or("")
                .to_string(),
            error: Some(error),
        }
    }
}

#[async_trait]
impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        if let Some(error) = self.error {
            error!(%error, "Error from controller");
        }

        let body = format!(
            "{{\"code\": {}, \"message\": \"{}\"}}",
            self.code, self.message
        );

        Response::build()
            .header(ContentType::JSON)
            .status(Status::new(self.code))
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

impl From<Status> for ApiError {
    fn from(s: Status) -> Self {
        ApiError {
            code: s.code,
            message: s.reason().unwrap_or("").to_string(),
            error: None,
        }
    }
}
