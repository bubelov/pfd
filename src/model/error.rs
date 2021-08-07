use color_eyre::Report;
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
pub struct Error {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing)]
    pub report: Option<Report>,
}

impl Error {
    pub fn short(status: Status) -> Error {
        Error {
            code: status.code,
            message: status.reason().unwrap().to_string(),
            report: None,
        }
    }

    pub fn full(status: Status, report: Report) -> Error {
        Error {
            code: status.code,
            message: status.reason().unwrap().to_string(),
            report: Some(report),
        }
    }
}

#[async_trait]
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        if let Some(report) = self.report {
            error!(%report, "Error from controller");
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
