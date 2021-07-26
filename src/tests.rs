use crate::{rocket, Db, model::ExchangeRate};
use rocket::{http::Status, local::asynchronous::Client, serde::json};

#[rocket::async_test]
async fn exchange_rates_controller_get() {
    let client = Client::untracked(rocket()).await.unwrap();
    let db = Db::get_one(client.rocket()).await.unwrap();
    let rate = ExchangeRate {
        base: "USD".to_string(),
        quote: "EUR".to_string(),
        rate: 1.25,
    };

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

    let req = client.get("/exchange_rates?base=USD&quote=EUR");
    let res = req.dispatch().await;
    assert_eq!(res.status(), Status::Ok);
    let body = res.into_string().await.unwrap();
    let body: ExchangeRate = json::from_str(&body).unwrap();
    assert_eq!(rate, body);

    let req = client.get("/exchange_rates?base=USD&quote=EUR");
    let res = req.dispatch().await;
    assert_eq!(res.status(), Status::Ok);
    // Time to halt forever...
    let body = res.into_json::<ExchangeRate>().await.unwrap();
    assert_eq!(rate, body);
}
