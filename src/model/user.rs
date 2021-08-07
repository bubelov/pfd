use crate::db::Db;
use crate::service::users;
use rocket::{
    http::Status,
    outcome::try_outcome,
    request::{FromRequest, Outcome, Request},
};

pub struct User {
    pub id: String,
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
            let id = auth_header[1];
            let user = users::get(id, db).await;

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
