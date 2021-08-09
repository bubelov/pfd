use super::ApiError;
use anyhow::{Error, Result};
use rocket::serde::Serialize;
use rocket::{http::Status, response::Responder, serde::json::Json};

#[derive(Responder)]
#[response(bound = "T: Serialize")]
pub enum ApiResult<T> {
    #[response(status = 200)]
    Ok(Json<T>),
    #[response(status = 201)]
    Created(Json<T>),
    Err(ApiError),
}

impl<T> ApiResult<T> {
    pub fn internal_error(e: Error) -> ApiResult<T> {
        ApiResult::Err(ApiError::full(Status::InternalServerError, e))
    }

    pub fn new(result: Result<Option<T>>) -> ApiResult<T> {
        match result {
            Ok(opt) => match opt {
                Some(val) => ApiResult::Ok(Json(val)),
                None => ApiResult::Err(ApiError::short(Status::NotFound)),
            },
            Err(e) => ApiResult::Err(ApiError::full(Status::InternalServerError, e)),
        }
    }
}
