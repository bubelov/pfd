use crate::model::{AuthToken, Id};
use anyhow::Error;
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::Mutex;

pub struct AuthTokenRepository {
    conn: Mutex<Connection>,
}

impl AuthTokenRepository {
    pub fn new(conn: Connection) -> AuthTokenRepository {
        AuthTokenRepository {
            conn: Mutex::new(conn),
        }
    }

    pub fn insert(&self, row: &AuthToken) -> anyhow::Result<()> {
        let query = "INSERT INTO auth_token (id, username) VALUES (?, ?)";
        let params = params![&row.id, &row.username];
        self.conn
            .lock()
            .unwrap()
            .execute(query, params)
            .map(|_| ())
            .map_err(|e| Error::new(e))
    }

    pub fn select_by_id(&self, id: &Id) -> anyhow::Result<Option<AuthToken>> {
        self.conn
            .lock()
            .unwrap()
            .query_row(
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
            .map_err(|e| Error::new(e))
    }
}

#[cfg(test)]
mod test {
    use crate::{model::AuthToken, repository::AuthTokenRepository, test::db};
    use anyhow::Result;
    use uuid::Uuid;

    #[test]
    fn insert() -> Result<()> {
        let repo = AuthTokenRepository::new(db());
        repo.insert(&token())?;
        Ok(())
    }

    #[test]
    fn select_by_id() -> Result<()> {
        let repo = AuthTokenRepository::new(db());
        let row = token();
        let res = repo.select_by_id(&row.id)?;
        assert!(res.is_none());
        repo.insert(&row)?;
        let res = repo.select_by_id(&row.id)?;
        assert_eq!(Some(row), res);
        Ok(())
    }

    fn token() -> AuthToken {
        AuthToken {
            id: Uuid::new_v4().into(),
            username: "test".into(),
        }
    }
}
