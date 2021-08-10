use crate::{
    db::Db,
    model::{ApiResult, AuthToken, Id, User},
    service::{auth_tokens, users},
};
use rocket::{post, serde::json::Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct PostInput {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct PostOutput {
    pub user: User,
    pub auth_token: AuthToken,
}

#[post("/users", data = "<input>")]
pub async fn post(input: Json<PostInput>, db: Db) -> ApiResult<PostOutput> {
    let user = User {
        username: input.username.clone(),
        password_hash: input.password.clone(),
    };

    if let Err(e) = users::insert_or_replace(&user, &db).await {
        return ApiResult::internal_error(e);
    }

    let auth_token = AuthToken {
        id: Id(Uuid::new_v4()),
        username: user.username.clone(),
    };

    if let Err(e) = auth_tokens::insert_or_replace(&auth_token, &db).await {
        return ApiResult::internal_error(e);
    }

    ApiResult::Created(Json(PostOutput {
        user: user,
        auth_token: auth_token,
    }))
}
