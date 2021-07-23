pub fn run(c: &mut rusqlite::Connection) {
    println!("Running migrations...");

    c.execute(
        r#"
        CREATE TABLE exchange_rate (
            base TEXT NOT NULL,
            quote TEXT NOT NULL,
            rate REAL NOT NULL
        )
        "#,
        []).unwrap();
}
