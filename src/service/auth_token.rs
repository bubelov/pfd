use crate::{
    db::Db,
    model::{AuthToken, Id},
    repository::auth_token,
};
use anyhow::{Error, Result};

pub async fn insert_or_replace(item: &AuthToken, db: &Db) -> Result<()> {
    let item = item.clone();
    db.run(move |conn| auth_token::insert_or_replace(&item, conn))
        .await
        .map(|_| ())
        .map_err(|e| Error::new(e))
}

pub async fn select_by_id(id: &Id, db: &Db) -> Result<Option<AuthToken>> {
    let id = id.clone();
    db.run(move |conn| auth_token::select_by_id(&id, conn))
        .await
        .map_err(|e| Error::new(e))
}
