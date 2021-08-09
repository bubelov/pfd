use crate::model::{AuthToken, Id};
use rusqlite::{params, Connection, OptionalExtension, Result};

pub fn insert_or_replace(row: &AuthToken, conn: &mut Connection) -> Result<usize> {
    let query = "INSERT OR REPLACE INTO auth_token (id, user_id) VALUES (?, ?)";
    let params = params![&row.id, &row.user_id];
    conn.execute(query, params)
}

pub fn select_by_id(id: &Id, conn: &mut Connection) -> Result<Option<AuthToken>> {
    conn.query_row(
        "SELECT id, user_id FROM auth_token WHERE id = ?",
        params![id],
        |row| {
            Ok(AuthToken {
                id: row.get(0)?,
                user_id: row.get(1)?,
            })
        },
    )
    .optional()
}
