use rocket::{
    async_trait,
    http::{ContentType, Status},
    request::Request,
    response::{self, Responder, Response},
    serde::{Deserialize, Serialize},
};
use std::io::Cursor;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ExchangeRate {
    pub quote: String,
    pub base: String,
    pub rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Error {
    pub code: u16,
    pub message: String,
}

impl Error {
    pub fn new(code: u16, message: &str) -> Error {
        Error {
            code: code,
            message: message.to_string(),
        }
    }
}

#[async_trait]
impl<'r> Responder<'r, 'static> for Error {
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
