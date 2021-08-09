use crate::{db::Db, model::User, repository::users};
use color_eyre::Report;
use uuid::Uuid;

pub async fn get_by_id(id: &Uuid, db: Db) -> Result<Option<User>, Report> {
    let id_owned = id.clone();
    db.run(move |conn| users::select_by_id(conn, &id_owned))
        .await
        .map_err(|e| Report::new(e))
}
