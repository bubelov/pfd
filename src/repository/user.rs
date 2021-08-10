use crate::model::User;
use rusqlite::{params, Connection, OptionalExtension, Result};

pub fn insert_or_replace(row: &User, conn: &mut Connection) -> Result<usize> {
    let query = "INSERT OR REPLACE INTO user (username, password_hash) VALUES (?, ?)";
    let params = params![&row.username, &row.password_hash];
    conn.execute(query, params)
}

pub fn select_by_username(username: &str, conn: &mut Connection) -> Result<Option<User>> {
    conn.query_row(
        "SELECT password_hash FROM user WHERE username = ?",
        params![username],
        |row| {
            Ok(User {
                username: username.into(),
                password_hash: row.get(0)?,
            })
        },
    )
    .optional()
}

#[cfg(test)]
mod test {
    use crate::{model::User, test::setup_db};
    use rusqlite::Result;

    #[test]
    fn insert_or_replace() -> Result<()> {
        let mut conn = setup_db();
        let row = user();
        assert_eq!(1, super::insert_or_replace(&row, &mut conn)?);
        Ok(())
    }

    #[test]
    fn select_by_username() -> Result<()> {
        let mut conn = setup_db();
        let row = user();
        let res = super::select_by_username(&row.username, &mut conn)?;
        assert!(res.is_none());
        super::insert_or_replace(&row, &mut conn)?;
        let res = super::select_by_username(&row.username, &mut conn)?;
        assert_eq!(Some(row), res);
        Ok(())
    }

    fn user() -> User {
        User {
            username: "test".into(),
            password_hash: "test".into(),
        }
    }
}
