#[rocket::async_test]
async fn get_exchange_rate() {
    let rocket = super::rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket).await.unwrap();
    let request = client.get("/exchange_rates?base=USD&quote=BTC");
    let response = request.dispatch().await;
    assert_eq!(response.status(), rocket::http::Status::Ok);
}
