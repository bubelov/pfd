use rocket::local::asynchronous::Client;

#[rocket::async_test]
async fn exchange_rates_controller_get() {
    let client = Client::untracked(crate::rocket()).await.unwrap();
    println!("Initialized test client");
    let db = crate::Db::get_one(client.rocket()).await.unwrap();
    println!("Got test DB handle");

    db.run(|c| {
        c.execute(
            r#"
            INSERT INTO exchange_rate (base, quote, rate)
            VALUES('USD', 'EUR', 1.25)
            "#,
            [],
        )
        .unwrap();
    })
    .await;

    // NOTE it returns "no table" error, which is unexpected
    // because migration should have created that table
    let request = client.get("/exchange_rates?base=USD&quote=EUR");
    let response = request.dispatch().await;

    assert_eq!(response.status(), rocket::http::Status::Ok);
}
