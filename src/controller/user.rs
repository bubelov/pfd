use crate::{
    model::{ApiError, ApiResult, AuthToken, Id, User},
    repository::{AuthTokenRepository, UserRepository},
    service::{auth_token, user},
};
use rand::RngCore;
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

#[post("/users", data = "<input>")]
pub async fn post(
    input: Json<PostInput>,
    user_repo: &State<UserRepository>,
    token_repo: &State<AuthTokenRepository>,
) -> ApiResult<PostOutput> {
    match user::select_by_username(&input.username, user_repo) {
        Ok(opt) => match opt {
            Some(_) => return ApiError::custom(400, "This username is already taken").into(),
            None => {}
        },
        Err(e) => return e.into(),
    }

    let mut salt = [0u8; 128];
    rand::thread_rng().fill_bytes(&mut salt);
    let argon2_config = argon2::Config::default();
    let password_hash =
        argon2::hash_encoded(input.password.as_bytes(), &salt, &argon2_config).unwrap();

    let user = User {
        username: input.username.clone(),
        password_hash: password_hash,
    };

    if let Err(e) = user::insert(&user, user_repo) {
        return e.into();
    }

    let auth_token = AuthToken {
        id: Id::new(),
        username: user.username.clone(),
    };

    if let Err(e) = auth_token::insert(&auth_token, token_repo) {
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
        controller::user::{PostInput, PostOutput},
        repository::UserRepository,
        test::client,
    };
    use anyhow::Result;
    use rocket::http::Status;

    #[test]
    fn post() -> Result<()> {
        let client = client();
        let input = PostInput {
            username: "test2".into(),
            password: "test2".into(),
        };
        let res = client.post("/users").json(&input).dispatch();
        assert_eq!(res.status(), Status::Created);
        res.into_json::<PostOutput>().unwrap();
        Ok(())
    }

    #[test]
    fn post_twice() -> Result<()> {
        let client = client();
        let input = PostInput {
            username: "test2".into(),
            password: "test2".into(),
        };
        let _res = client.post("/users").json(&input).dispatch();
        let res = client.post("/users").json(&input).dispatch();
        assert_eq!(res.status(), Status::BadRequest);
        Ok(())
    }

    #[test]
    fn post_password_hashing() -> Result<()> {
        let client = client();
        let input = PostInput {
            username: "test2".into(),
            password: "test2".into(),
        };
        client.post("/users").json(&input).dispatch();
        let user_repo = client.rocket().state::<UserRepository>().unwrap();
        let user = user_repo.select_by_username(&input.password)?.unwrap();
        assert_ne!(
            user.password_hash, input.password,
            "Password was stored in plaintext!"
        );
        let matches =
            argon2::verify_encoded(&user.password_hash, input.password.as_bytes()).unwrap();
        assert!(matches);
        Ok(())
    }
}
