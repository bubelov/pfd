use crate::model::ExchangeRate;
use rusqlite::{params, Connection, Error, OptionalExtension};

#[allow(dead_code)]
pub fn insert_or_replace(conn: &mut Connection, row: &ExchangeRate) -> Result<usize, Error> {
    let query = "INSERT OR REPLACE INTO exchange_rate (quote, base, rate) VALUES (?, ?, ?)";
    let params = params![&row.quote, &row.base, row.rate];
    conn.execute(query, params)
}

pub fn select_by_quote_and_base(
    conn: &mut Connection,
    quote: &str,
    base: &str,
) -> Result<Option<ExchangeRate>, Error> {
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
