use super::Error;
use color_eyre::Report;
use rocket::serde::Serialize;
use rocket::{http::Status, response::Responder, serde::json::Json};

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
