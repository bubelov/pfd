pub fn find_by_base_and_quote(
    conn: &mut rusqlite::Connection, 
    base: String, 
    quote: String,
) -> Option<crate::model::ExchangeRate> {
    let rate = conn.query_row(
        "SELECT rate FROM exchange_rate WHERE base = :base AND quote = :quote",
        rusqlite::named_params!{":base": base.clone(), ":quote": quote.clone()},
        |r| {
        Ok(crate::model::ExchangeRate {
            base: base.clone(),
            quote: quote.clone(),
            rate: r.get(0)?
        })
    }).unwrap();

    Some(rate)
}
