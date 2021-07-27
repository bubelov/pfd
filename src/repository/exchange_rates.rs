use crate::model::ExchangeRate;
use rusqlite::{params, Connection, Error};

pub fn insert(conn: &mut Connection, row: &ExchangeRate) {
    let query = "INSERT INTO exchange_rate (base, quote, rate) VALUES (?, ?, ?)";
    let params = params![&row.base, &row.quote, row.rate];
    conn.execute(query, params).unwrap();
}

pub fn select_by_base_and_quote(
    conn: &mut Connection,
    base: &String,
    quote: &String,
) -> Result<ExchangeRate, Error> {
    conn.query_row(
        "SELECT rate FROM exchange_rate WHERE base = ? AND quote = ?",
        params![base, quote],
        |r| {
            Ok(ExchangeRate {
                base: base.clone(),
                quote: quote.clone(),
                rate: r.get(0)?,
            })
        },
    )
}
