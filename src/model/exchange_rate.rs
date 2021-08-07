use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ExchangeRate {
    pub quote: String,
    pub base: String,
    pub rate: f64,
}
