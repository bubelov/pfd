use crate::{model::ExchangeRate, repository::exchange_rates, rocket, Db};
use rocket::{http::Status, local::asynchronous::Client, serde::json};

async fn setup() -> (Client, Db) {
    let profile = rocket::Config::figment().select("test");
    let rocket = rocket(rocket::custom(&profile)).await;
    let client = Client::untracked(rocket).await.unwrap();
    let db = Db::get_one(client.rocket()).await.unwrap();
    (client, db)
}

#[rocket::async_test]
async fn exchange_rates_controller_get() {
    let (client, db) = setup().await;

    fn rate() -> ExchangeRate {
        ExchangeRate {
            base: "USD".to_string(),
            quote: "EUR".to_string(),
            rate: 1.25,
        }
    }

    db.run(|conn| exchange_rates::insert(conn, &rate())).await;

    let res = client
        .get("/exchange_rates?base=USD&quote=EUR")
        .dispatch()
        .await;
    assert_eq!(res.status(), Status::Ok);

    let body = res.into_string().await.unwrap();
    let body: ExchangeRate = json::from_str(&body).unwrap();
    assert_eq!(rate(), body);
}
