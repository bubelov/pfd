use crate::{db::Db, model::User, repository::user};
use anyhow::{Error, Result};

pub async fn insert(user: &User, db: &Db) -> Result<()> {
    let user = user.clone();
    db.run(move |conn| user::insert(&user, conn))
        .await
        .map(|_| ())
        .map_err(|e| Error::new(e))
}

pub async fn select_by_username(username: &str, db: &Db) -> Result<Option<User>> {
    let username = username.to_string();
    db.run(move |conn| user::select_by_username(&username, conn))
        .await
        .map_err(|e| Error::new(e))
}
