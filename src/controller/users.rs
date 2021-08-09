use crate::{
    db::Db,
    model::{ApiResult, AuthToken, Id, User},
    service::{auth_tokens, users},
};
use rocket::{post, serde::json::Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct PostPayload {
    user: User,
    auth_token: AuthToken,
}

#[post("/users")]
pub async fn post(db: Db) -> ApiResult<PostPayload> {
    let user = User {
        id: Id(Uuid::new_v4()),
    };

    if let Err(e) = users::insert_or_replace(&user, &db).await {
        return ApiResult::internal_error(e);
    }

    let auth_token = AuthToken {
        id: Id(Uuid::new_v4()),
        user_id: user.id.clone(),
    };

    if let Err(e) = auth_tokens::insert_or_replace(&auth_token, &db).await {
        return ApiResult::internal_error(e);
    }

    ApiResult::Created(Json(PostPayload {
        user: user,
        auth_token: auth_token,
    }))
}
