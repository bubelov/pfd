use crate::model::ApiError;
use anyhow::{Error, Result};
use rocket::{http::Status, response::Responder, serde::json::Json};
use serde::Serialize;

#[derive(Responder)]
#[response(bound = "T: Serialize")]
pub enum ApiResult<T> {
    Ok((Status, Json<T>)),
    Err((Status, Json<ApiError>)),
}

impl<T> ApiResult<T> {
    pub fn new(code: u16, t: T) -> ApiResult<T> {
        ApiResult::Ok((Status::from_code(code).unwrap(), Json(t)))
    }
}

impl<T> From<ApiError> for ApiResult<T> {
    fn from(e: ApiError) -> Self {
        ApiResult::Err((Status::from_code(e.code).unwrap(), Json(e)))
    }
}

impl<T> From<Error> for ApiResult<T> {
    fn from(_: Error) -> Self {
        ApiError::new(500).into()
    }
}

impl<T> From<Result<Option<T>>> for ApiResult<T> {
    fn from(r: Result<Option<T>>) -> Self {
        match r {
            Ok(opt) => match opt {
                Some(val) => ApiResult::new(200, val),
                None => ApiError::new(404).into(),
            },
            Err(e) => e.into(),
        }
    }
}
