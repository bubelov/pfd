use crate::{
    db::Db,
    model::{Id, User},
    repository::users,
};
use anyhow::{Error, Result};

pub async fn insert_or_replace(user: &User, db: &Db) -> Result<()> {
    let user = user.clone();
    db.run(move |conn| users::insert_or_replace(&user, conn))
        .await
        .map(|_| ())
        .map_err(|e| Error::new(e))
}

pub async fn select_by_id(id: &Id, db: &Db) -> Result<Option<User>> {
    let id_owned = id.clone();
    db.run(move |conn| users::select_by_id(&id_owned, conn))
        .await
        .map_err(|e| Error::new(e))
}
