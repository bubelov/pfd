use crate::{
    db::Db,
    model::Id,
    service::{auth_token, user},
};
use rocket::{
    http::Status,
    outcome::try_outcome,
    request::{FromRequest, Outcome, Request},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password_hash: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers: Vec<_> = req.headers().get("Authorization").collect();

        if headers.len() == 0 {
            return Outcome::Failure((Status::Unauthorized, ()));
        }

        if headers.len() > 1 {
            return Outcome::Failure((Status::BadRequest, ()));
        }

        let header = headers.get(0);

        if let None = header {
            return Outcome::Failure((Status::Unauthorized, ()));
        }

        let value_parts: Vec<_> = header.unwrap().split(" ").collect();

        if value_parts.len() != 2 {
            return Outcome::Failure((Status::BadRequest, ()));
        }

        let auth_type = value_parts[0];
        let auth_credentials = value_parts[1];

        if auth_type != "Bearer" {
            return Outcome::Failure((Status::BadRequest, ()));
        }

        let db: Db = try_outcome!(req.guard::<Db>().await);

        let token_id = auth_credentials.parse::<Id>();

        if let Err(_) = token_id {
            return Outcome::Failure((Status::BadRequest, ()));
        }

        let token = auth_token::select_by_id(&token_id.unwrap(), &db).await;

        if let Err(_) = token {
            return Outcome::Failure((Status::InternalServerError, ()));
        }

        let token = token.unwrap();

        if let None = token {
            return Outcome::Failure((Status::Unauthorized, ()));
        }

        return match user::select_by_username(&token.unwrap().username, &db).await {
            Ok(user) => match user {
                Some(user) => Outcome::Success(user),
                None => Outcome::Failure((Status::BadRequest, ())),
            },
            Err(_e) => Outcome::Failure((Status::BadRequest, ())),
        };
    }
}
