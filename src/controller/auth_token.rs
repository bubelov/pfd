use crate::{
    db::Db,
    model::{ApiError, ApiResult, AuthToken, Id, User},
    service::{auth_token, user},
};
use rocket::{post, serde::json::Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PostInput {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub struct PostOutput {
    user: UserView,
    auth_token: AuthTokenView,
}

#[derive(Serialize, Deserialize)]
pub struct UserView {
    username: String,
}

impl From<User> for UserView {
    fn from(user: User) -> UserView {
        UserView {
            username: user.username.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AuthTokenView {
    id: Id,
}

impl From<AuthToken> for AuthTokenView {
    fn from(token: AuthToken) -> AuthTokenView {
        AuthTokenView {
            id: token.id.clone(),
        }
    }
}

#[post("/auth_tokens", data = "<input>")]
pub async fn post(input: Json<PostInput>, db: Db) -> ApiResult<PostOutput> {
    let user = user::select_by_username(&input.username, &db).await;

    if let Err(e) = user {
        return e.into();
    }

    let user = user.unwrap();

    if let None = user {
        let message = format!("Username {} doesn't exist", input.username);
        return ApiError::custom(400, &message).into();
    }

    let user = user.unwrap();

    let matches = argon2::verify_encoded(&user.password_hash, input.password.as_bytes()).unwrap();

    if !matches {
        return ApiError::custom(400, "Invalid password").into();
    }

    let auth_token = AuthToken {
        id: Id::new(),
        username: user.username.clone(),
    };

    if let Err(e) = auth_token::insert_or_replace(&auth_token, &db).await {
        return e.into();
    }

    ApiResult::new(
        201,
        PostOutput {
            user: user.into(),
            auth_token: auth_token.into(),
        },
    )
}

#[cfg(test)]
mod test {
    use crate::test::setup;
    use anyhow::Result;
    use rocket::http::Status;

    #[test]
    fn post() -> Result<()> {
        let (client, _) = setup();
        let input = super::PostInput {
            username: "test".into(),
            password: "test".into(),
        };
        let res = client.post("/auth_tokens").json(&input).dispatch();
        assert_eq!(res.status(), Status::Created);
        res.into_json::<super::PostOutput>().unwrap();
        Ok(())
    }
}
