pub fn run(c: &mut rusqlite::Connection) {
    c.execute(
        r#"
        CREATE TABLE exchange_rate (
            base TEXT NOT NULL,
            quote TEXT NOT NULL,
            rate REAL NOT NULL
        )
        "#,
        []).unwrap();

    c.execute(
        r#"
        INSERT INTO exchange_rate (base, quote, rate)
        VALUES('USD', 'BTC', 35000)
        "#,
        []).unwrap();
}
