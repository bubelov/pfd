use crate::model::User;
use rusqlite::{params, Connection, OptionalExtension, Result};

pub fn insert_or_replace(row: &User, conn: &mut Connection) -> Result<usize> {
    let query = "INSERT OR REPLACE INTO user (username, password_hash) VALUES (?, ?)";
    let params = params![&row.username, &row.password_hash];
    conn.execute(query, params)
}

pub fn select_by_username(username: &String, conn: &mut Connection) -> Result<Option<User>> {
    conn.query_row(
        "SELECT password_hash FROM user WHERE username = ?",
        params![username],
        |row| {
            Ok(User {
                username: username.clone(),
                password_hash: row.get(0)?,
            })
        },
    )
    .optional()
}
