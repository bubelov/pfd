use rusqlite::Connection;

pub fn run(conn: &mut Connection) {
    conn.execute(
        r#"
        CREATE TABLE exchange_rate (
            base TEXT NOT NULL,
            quote TEXT NOT NULL,
            rate REAL NOT NULL
        )
        "#,
        [],
    )
    .unwrap();
}
