use crate::{
    db::Db,
    model::{ApiResult, AuthToken, Id, User},
    service::{auth_token, user},
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

    if let Err(e) = user::insert_or_replace(&user, &db).await {
        return ApiResult::internal_error(e);
    }

    let auth_token = AuthToken {
        id: Id(Uuid::new_v4()),
        username: user.username.clone(),
    };

    if let Err(e) = auth_token::insert_or_replace(&auth_token, &db).await {
        return ApiResult::internal_error(e);
    }

    ApiResult::Created(Json(PostOutput {
        user: user,
        auth_token: auth_token,
    }))
}

#[cfg(test)]
mod test {
    use crate::test::setup_without_auth;
    use rocket::http::Status;

    #[test]
    fn post() {
        let client = setup_without_auth();
        let input = super::PostInput {
            username: "test".to_string(),
            password: "test".to_string(),
        };
        let res = client.post("/users").json(&input).dispatch();
        assert_eq!(res.status(), Status::Created);
        res.into_json::<super::PostOutput>().unwrap();
    }
}
