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
        let auth_headers: Vec<_> = req.headers().get("Authorization").collect();
        let db: Db = try_outcome!(req.guard::<Db>().await);

        if auth_headers.len() != 1 {
            return Outcome::Failure((Status::Unauthorized, ()));
        }

        let auth_header: Vec<_> = auth_headers[0].split(" ").collect();

        if auth_header.len() == 2 {
            let token_id = auth_header[1].parse::<Id>().unwrap();
            let token = auth_token::select_by_id(&token_id, &db)
                .await
                .unwrap()
                .unwrap();
            let user = user::select_by_username(&token.username, &db).await;

            return match user {
                Ok(user) => match user {
                    Some(user) => Outcome::Success(user),
                    None => Outcome::Failure((Status::BadRequest, ())),
                },
                Err(_e) => Outcome::Failure((Status::BadRequest, ())),
            };
        }

        Outcome::Failure((Status::BadRequest, ()))
    }
}
