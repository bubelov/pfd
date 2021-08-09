use crate::model::{Id, User};
use rusqlite::{params, Connection, Error, OptionalExtension};

pub fn insert_or_replace(row: &User, conn: &mut Connection) -> Result<usize, Error> {
    let query = "INSERT OR REPLACE INTO user (id) VALUES (?)";
    let params = params![&row.id];
    conn.execute(query, params)
}

pub fn select_by_id(id: &Id, conn: &mut Connection) -> Result<Option<User>, Error> {
    conn.query_row("SELECT id FROM user WHERE id = ?", params![id], |row| {
        Ok(User { id: row.get(0)? })
    })
    .optional()
}
