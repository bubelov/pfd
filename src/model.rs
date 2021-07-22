#[derive(Debug, rocket::serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ExchangeRate {
    pub base: String,
    pub quote: String,
    pub rate: f64,
}

#[derive(Debug, rocket::serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ErrorResponseBody {
    pub code: u16,
    pub message: String,
}
