use crate::{
    model::{ApiError, ApiResult, AuthToken, Id, User},
    repository::{AuthTokenRepository, UserRepository},
    service::{auth_token, user},
};
use rocket::{post, serde::json::Json, State};
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
pub async fn post(
    input: Json<PostInput>,
    auth_token_repo: &State<AuthTokenRepository>,
    user_repo: &State<UserRepository>,
) -> ApiResult<PostOutput> {
    let user = user::select_by_username(&input.username, user_repo);

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

    if let Err(e) = auth_token::insert(&auth_token, auth_token_repo) {
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
    use crate::{
        controller::auth_token::{PostInput, PostOutput},
        test::client,
    };
    use anyhow::Result;
    use rocket::http::Status;

    #[test]
    fn post() -> Result<()> {
        let client = client();
        let input = PostInput {
            username: "test".into(),
            password: "test".into(),
        };
        let res = client.post("/auth_tokens").json(&input).dispatch();
        assert_eq!(res.status(), Status::Created);
        res.into_json::<PostOutput>().unwrap();
        Ok(())
    }
}
