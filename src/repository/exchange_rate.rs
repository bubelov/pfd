use crate::model::ExchangeRate;
use rusqlite::{params, Connection, OptionalExtension, Result};

pub fn insert_or_replace(row: &ExchangeRate, conn: &mut Connection) -> Result<usize> {
    let query = "INSERT OR REPLACE INTO exchange_rate (quote, base, rate) VALUES (?, ?, ?)";
    let params = params![&row.quote, &row.base, row.rate];
    conn.execute(query, params)
}

pub fn select_by_quote_and_base(
    quote: &str,
    base: &str,
    conn: &mut Connection,
) -> Result<Option<ExchangeRate>> {
    conn.query_row(
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
}

#[cfg(test)]
mod test {
    use crate::{model::ExchangeRate, test::setup_db};
    use rusqlite::Result;

    #[test]
    fn insert_or_replace() -> Result<()> {
        let mut conn = setup_db();
        let row = exchange_rate();
        assert_eq!(1, super::insert_or_replace(&row, &mut conn)?);
        Ok(())
    }

    #[test]
    fn select_by_quote_and_base() -> Result<()> {
        let mut conn = setup_db();
        let row = exchange_rate();
        let res = super::select_by_quote_and_base(&row.quote, &row.base, &mut conn)?;
        assert!(res.is_none());
        super::insert_or_replace(&row, &mut conn)?;
        let res = super::select_by_quote_and_base(&row.quote, &row.base, &mut conn)?;
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
