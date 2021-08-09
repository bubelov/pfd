use crate::model::User;
use rusqlite::{params, Connection, Error, OptionalExtension};
use uuid::Uuid;

#[allow(dead_code)]
pub fn insert_or_replace(conn: &mut Connection, row: &User) -> Result<usize, Error> {
    let query = "INSERT OR REPLACE INTO user (id) VALUES (?)";
    let params = params![&row.id];
    conn.execute(query, params)
}

#[allow(dead_code)]
pub fn select_by_id(conn: &mut Connection, id: &Uuid) -> Result<Option<User>, Error> {
    conn.query_row("SELECT id FROM user WHERE id = ?", params![id], |row| {
        Ok(User { id: row.get(0)? })
    })
    .optional()
}
