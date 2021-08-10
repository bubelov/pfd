use crate::model::{AuthToken, Id};
use rusqlite::{params, Connection, OptionalExtension, Result};

pub fn insert_or_replace(row: &AuthToken, conn: &mut Connection) -> Result<usize> {
    let query = "INSERT OR REPLACE INTO auth_token (id, username) VALUES (?, ?)";
    let params = params![&row.id, &row.username];
    conn.execute(query, params)
}

pub fn select_by_id(id: &Id, conn: &mut Connection) -> Result<Option<AuthToken>> {
    conn.query_row(
        "SELECT id, username FROM auth_token WHERE id = ?",
        params![id],
        |row| {
            Ok(AuthToken {
                id: row.get(0)?,
                username: row.get(1)?,
            })
        },
    )
    .optional()
}

#[cfg(test)]
mod test {
    use crate::{model::AuthToken, test::setup_db};
    use rusqlite::Result;
    use uuid::Uuid;

    #[test]
    fn insert_or_replace() -> Result<()> {
        let mut conn = setup_db();
        let row = auth_token();
        assert_eq!(1, super::insert_or_replace(&row, &mut conn)?);
        Ok(())
    }

    #[test]
    fn select_by_id() -> Result<()> {
        let mut conn = setup_db();
        let row = auth_token();
        let res = super::select_by_id(&row.id, &mut conn)?;
        assert!(res.is_none());
        super::insert_or_replace(&row, &mut conn)?;
        let res = super::select_by_id(&row.id, &mut conn)?;
        assert_eq!(Some(row), res);
        Ok(())
    }

    fn auth_token() -> AuthToken {
        AuthToken {
            id: Uuid::new_v4().into(),
            username: "test".into(),
        }
    }
}
