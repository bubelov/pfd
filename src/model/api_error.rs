use rocket::{
    async_trait,
    http::{ContentType, Status},
    request::Request,
    response::{self, Responder, Response},
};
use serde::Serialize;
use std::io::Cursor;

#[derive(Serialize)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
}

impl ApiError {
    pub fn new(code: u16) -> ApiError {
        ApiError {
            code: code,
            message: Status::from_code(code)
                .unwrap()
                .reason()
                .unwrap_or("")
                .to_string(),
        }
    }
}

#[async_trait]
impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
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
        }
    }
}
