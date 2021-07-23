use rocket::local::asynchronous::Client;

#[rocket::async_test]
async fn exchange_rates_controller_get() {
    let client = Client::untracked(crate::rocket()).await.unwrap();
    //let db = crate::Db::get_one(client.rocket()).await.unwrap();

    //db.run(|c| {
    //    c.execute(
    //        r#"
    //        INSERT INTO exchange_rate (base, quote, rate)
    //        VALUES('USD', 'EUR', 1.25)
    //        "#,
    //        []
    //    ).unwrap();
    //}).await;

    let request = client.get("/exchange_rates?base=USD&quote=EUR");
    let response = request.dispatch().await;

    assert_eq!(response.status(), rocket::http::Status::Ok);
}
