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
            message: s.reason().unwrap().to_string(),
            error: None,
        })
    }
}

impl<T> From<(Status, Error)> for ApiResult<T> {
    fn from(t: (Status, Error)) -> Self {
        ApiResult::Err(ApiError::new(t.0.code, t.1))
    }
}

impl<T> From<(u16, Error)> for ApiResult<T> {
    fn from(t: (u16, Error)) -> Self {
        ApiResult::Err(ApiError::new(t.0, t.1))
    }
}

impl<T> From<Result<Option<T>>> for ApiResult<T> {
    fn from(r: Result<Option<T>>) -> Self {
        match r {
            Ok(opt) => match opt {
                Some(val) => ApiResult::ok(val),
                None => Status::NotFound.into(),
            },
            Err(e) => ApiError::new(500, e).into(),
        }
    }
}
