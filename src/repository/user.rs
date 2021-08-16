use crate::model::User;
use anyhow::{Error, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::Mutex;

pub struct UserRepository {
    conn: Mutex<Connection>,
}

impl UserRepository {
    pub fn new(conn: Connection) -> UserRepository {
        UserRepository {
            conn: Mutex::new(conn),
        }
    }

    pub fn insert(&self, row: &User) -> Result<()> {
        let row = row.clone();
        let query = "INSERT INTO user (username, password_hash) VALUES (?, ?)";
        let params = params![&row.username, &row.password_hash];
        self.conn
            .lock()
            .unwrap()
            .execute(query, params)
            .map(|_| ())
            .map_err(|e| Error::new(e))
    }

    pub fn select_by_username(&self, username: &str) -> anyhow::Result<Option<User>> {
        let username = username.to_string();
        self.conn
            .lock()
            .unwrap()
            .query_row(
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
            .map_err(|e| Error::new(e))
    }
}

#[cfg(test)]
mod test {
    use crate::{model::User, repository::UserRepository, test::setup_db};
    use anyhow::Result;

    #[test]
    fn insert() -> Result<()> {
        let repo = UserRepository::new(setup_db());
        repo.insert(&user())?;
        Ok(())
    }

    #[test]
    fn select_by_username() -> Result<()> {
        let repo = UserRepository::new(setup_db());
        let row = user();
        let res = repo.select_by_username(&row.username)?;
        assert!(res.is_none());
        repo.insert(&row)?;
        let res = repo.select_by_username(&row.username)?;
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
