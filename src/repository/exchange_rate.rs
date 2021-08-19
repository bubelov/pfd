use crate::model::ExchangeRate;
use anyhow::Error;
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::Mutex;

pub struct ExchangeRateRepository {
    conn: Mutex<Connection>,
}

impl ExchangeRateRepository {
    pub fn new(conn: Connection) -> ExchangeRateRepository {
        ExchangeRateRepository {
            conn: Mutex::new(conn),
        }
    }

    pub fn insert_or_replace(&self, row: &ExchangeRate) -> anyhow::Result<()> {
        let query = "INSERT OR REPLACE INTO exchange_rate (quote, base, rate) VALUES (?, ?, ?)";
        let params = params![&row.quote, &row.base, row.rate];
        self.conn
            .lock()
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
        self.conn
            .lock()
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
    use crate::{model::ExchangeRate, repository::ExchangeRateRepository, test::setup_db};

    #[test]
    fn insert_or_replace() -> anyhow::Result<()> {
        let repo = ExchangeRateRepository::new(setup_db());
        repo.insert_or_replace(&exchange_rate())?;
        Ok(())
    }

    #[test]
    fn select_by_quote_and_base() -> anyhow::Result<()> {
        let repo = ExchangeRateRepository::new(setup_db());
        let row = exchange_rate();
        let res = repo.select_by_quote_and_base(&row.quote, &row.base)?;
        assert!(res.is_none());
        repo.insert_or_replace(&row)?;
        let res = repo.select_by_quote_and_base(&row.quote, &row.base)?;
        assert_eq!(Some(row), res);
        Ok(())
    }

    fn exchange_rate() -> ExchangeRate {
        ExchangeRate {
            quote: "TST".into(),
            base: "TST".into(),
            rate: 1.0,
        }
    }
}
