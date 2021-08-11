use crate::model::ApiError;
use anyhow::{Error, Result};
use rocket::{http::Status, response::Responder, serde::json::Json};
use serde::Serialize;

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
    pub fn ok(t: T) -> ApiResult<T> {
        ApiResult::Ok(Json(t))
    }

    pub fn created(t: T) -> ApiResult<T> {
        ApiResult::Created(Json(t))
    }
}

impl<T> From<ApiError> for ApiResult<T> {
    fn from(e: ApiError) -> Self {
        ApiResult::Err(e)
    }
}

impl<T> From<Status> for ApiResult<T> {
    fn from(s: Status) -> Self {
        ApiResult::Err(ApiError {
            code: s.code,
            message: s.reason().unwrap_or("").to_string(),
            error: None,
        })
    }
}

impl<T> From<Error> for ApiResult<T> {
    fn from(e: Error) -> Self {
        ApiResult::Err(ApiError::new(500, e))
    }
}

impl<T> From<Result<Option<T>>> for ApiResult<T> {
    fn from(r: Result<Option<T>>) -> Self {
        match r {
            Ok(opt) => match opt {
                Some(val) => ApiResult::ok(val),
                None => Status::NotFound.into(),
            },
            Err(e) => e.into(),
        }
    }
}
