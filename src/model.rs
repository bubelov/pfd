use crate::db::Db;
use crate::service::users;
use color_eyre::Report;
use rocket::{
    async_trait,
    http::{ContentType, Status},
    outcome::try_outcome,
    request::{FromRequest, Outcome, Request},
    response::{self, Responder, Response},
    serde::{
        json::Json,
        {Deserialize, Serialize},
    },
};
use std::io::Cursor;
use tracing::error;

pub struct User {
    pub id: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_headers: Vec<_> = req.headers().get("Authorization").collect();
        let db: Db = try_outcome!(req.guard::<Db>().await);

        if auth_headers.len() != 1 {
            return Outcome::Failure((Status::Unauthorized, ()));
        }

        let auth_header: Vec<_> = auth_headers[0].split(" ").collect();

        if auth_header.len() == 2 {
            let id = auth_header[1];
            let user = users::get(id, db).await;

            return match user {
                Ok(user) => match user {
                    Some(user) => Outcome::Success(user),
                    None => Outcome::Failure((Status::BadRequest, ())),
                },
                Err(_e) => Outcome::Failure((Status::BadRequest, ())),
            };
        }

        Outcome::Failure((Status::BadRequest, ()))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ExchangeRate {
    pub quote: String,
    pub base: String,
    pub rate: f64,
}

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

#[derive(Responder)]
#[response(bound = "T: Serialize")]
pub enum ApiResult<T> {
    Ok(Json<T>),
    Err(Error),
}

impl<T> ApiResult<T> {
    pub fn new(result: Result<Option<T>, Report>) -> ApiResult<T> {
        match result {
            Ok(opt) => match opt {
                Some(val) => ApiResult::Ok(Json(val)),
                None => ApiResult::Err(Error::short(Status::NotFound)),
            },
            Err(e) => ApiResult::Err(Error::full(Status::InternalServerError, e)),
        }
    }
}
