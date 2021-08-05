use crate::{db::Db, model::User, repository::users};
use rusqlite::Error;

pub async fn get(id: &str, db: Db) -> Result<Option<User>, Error> {
    let id_owned = id.to_string();
    db.run(move |conn| users::select_by_id(conn, &id_owned))
        .await
}
