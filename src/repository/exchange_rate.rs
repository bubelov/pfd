use crate::model::ExchangeRate;
use anyhow::Error;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, OptionalExtension};

#[derive(Clone)]
pub struct ExchangeRateRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl ExchangeRateRepository {
    pub fn new(pool: &Pool<SqliteConnectionManager>) -> ExchangeRateRepository {
        ExchangeRateRepository { pool: pool.clone() }
    }

    pub fn insert_or_replace(&self, row: &ExchangeRate) -> anyhow::Result<()> {
        let query = "INSERT OR REPLACE INTO exchange_rate (quote, base, rate) VALUES (?, ?, ?)";
        let params = params![&row.quote, &row.base, row.rate];
        self.pool
            .get()
            .unwrap()
            .execute(query, params)
            .map(|_| ())
            .map_err(|e| Error::new(e))
    }

    pub fn select_by_quote_and_base(
        &self,
        quote: &str,
        base: &str,
    ) -> anyhow::Result<Option<ExchangeRate>> {
        self.pool
            .get()
            .unwrap()
            .query_row(
                "SELECT rate FROM exchange_rate WHERE quote = ? AND base = ?",
                params![quote, base],
                |row| {
                    Ok(ExchangeRate {
                        quote: quote.to_string(),
                        base: base.to_string(),
                        rate: row.get(0)?,
                    })
                },
            )
            .optional()
            .map_err(|e| Error::new(e))
    }
}

#[cfg(test)]
mod test {
    use crate::{model::ExchangeRate, repository::ExchangeRateRepository, test::pool};
    use anyhow::Result;

    #[test]
    fn insert_or_replace() -> Result<()> {
        let repo = ExchangeRateRepository::new(&pool());
        repo.insert_or_replace(&rate())?;
        Ok(())
    }

    #[test]
    fn select_by_quote_and_base() -> Result<()> {
        let repo = ExchangeRateRepository::new(&pool());
        let row = rate();
        let res = repo.select_by_quote_and_base(&row.quote, &row.base)?;
        assert!(res.is_none());
        repo.insert_or_replace(&row)?;
        let res = repo.select_by_quote_and_base(&row.quote, &row.base)?;
        assert_eq!(Some(row), res);
        Ok(())
    }

    fn rate() -> ExchangeRate {
        ExchangeRate {
            quote: "TST".into(),
            base: "TST".into(),
            rate: 1.0,
        }
    }
}
