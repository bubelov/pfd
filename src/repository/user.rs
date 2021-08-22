use crate::model::User;
use anyhow::{Error, Result};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, OptionalExtension};

#[derive(Clone)]
pub struct UserRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl UserRepository {
    pub fn new(pool: &Pool<SqliteConnectionManager>) -> UserRepository {
        UserRepository { pool: pool.clone() }
    }

    pub fn insert(&self, row: &User) -> Result<()> {
        let row = row.clone();
        let query = "INSERT INTO user (username, password_hash) VALUES (?, ?)";
        let params = params![&row.username, &row.password_hash];
        self.pool
            .get()
            .unwrap()
            .execute(query, params)
            .map(|_| ())
            .map_err(|e| Error::new(e))
    }

    pub fn select_by_username(&self, username: &str) -> anyhow::Result<Option<User>> {
        let username = username.to_string();
        self.pool
            .get()
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
    use crate::{model::User, repository::UserRepository, test::pool};
    use anyhow::Result;

    #[test]
    fn insert() -> Result<()> {
        let repo = UserRepository::new(&pool());
        repo.insert(&user())?;
        Ok(())
    }

    #[test]
    fn select_by_username() -> Result<()> {
        let repo = UserRepository::new(&pool());
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
