use crate::{db::Db, model::User, repository::users};
use color_eyre::Report;

pub async fn get(id: &str, db: Db) -> Result<Option<User>, Report> {
    let id_owned = id.to_string();
    db.run(move |conn| users::select_by_id(conn, &id_owned))
        .await
        .map_err(|e| Report::new(e))
}
