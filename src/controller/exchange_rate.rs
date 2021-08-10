use crate::{
    db::Db,
    model::{ApiResult, ExchangeRate, User},
    service::exchange_rate,
};
use rocket::get;

#[get("/exchange_rates?<quote>&<base>")]
pub async fn get(quote: &str, base: &str, db: Db, _user: User) -> ApiResult<ExchangeRate> {
    ApiResult::new(exchange_rate::get_by_quote_and_base(quote, base, db).await)
}

#[cfg(test)]
mod test {
    use crate::{
        model::ExchangeRate,
        repository::exchange_rate,
        test::{setup, setup_without_auth},
    };
    use rocket::http::Status;

    #[test]
    fn get() {
        let (client, mut db) = setup();

        let rate = ExchangeRate {
            quote: "EUR".to_string(),
            base: "USD".to_string(),
            rate: 1.25,
        };

        exchange_rate::insert_or_replace(&mut db, &rate).unwrap();

        let res = client.get("/exchange_rates?quote=EUR&base=USD").dispatch();

        assert_eq!(res.status(), Status::Ok);
        let body = res.into_json::<ExchangeRate>().unwrap();
        assert_eq!(rate, body);
    }

    #[test]
    fn get_inversed() {
        let (client, mut db) = setup();

        let rate = ExchangeRate {
            quote: "EUR".to_string(),
            base: "USD".to_string(),
            rate: 1.19,
        };

        let inversed_rate = ExchangeRate {
            quote: "USD".to_string(),
            base: "EUR".to_string(),
            rate: 1.0 / 1.19,
        };

        exchange_rate::insert_or_replace(&mut db, &rate).unwrap();

        let res = client.get("/exchange_rates?quote=USD&base=EUR").dispatch();

        assert_eq!(res.status(), Status::Ok);
        let body = res.into_json::<ExchangeRate>().unwrap();
        assert_eq!(inversed_rate, body);
    }

    #[test]
    fn get_indirect() {
        let (client, mut db) = setup();

        let usd_eur = ExchangeRate {
            quote: "USD".to_string(),
            base: "EUR".to_string(),
            rate: 0.840972163821378,
        };

        let rub_eur = ExchangeRate {
            quote: "RUB".to_string(),
            base: "EUR".to_string(),
            rate: 0.0115324823898994,
        };

        exchange_rate::insert_or_replace(&mut db, &usd_eur).unwrap();
        exchange_rate::insert_or_replace(&mut db, &rub_eur).unwrap();

        let rub_usd = ExchangeRate {
            quote: "RUB".to_string(),
            base: "USD".to_string(),
            rate: 0.0115324823898994 / 0.840972163821378,
        };

        let res = client.get("/exchange_rates?quote=RUB&base=USD").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.into_json::<ExchangeRate>().unwrap();
        assert_eq!(rub_usd, body);

        let usd_rub = ExchangeRate {
            quote: "USD".to_string(),
            base: "RUB".to_string(),
            rate: 0.840972163821378 / 0.0115324823898994,
        };

        let res = client.get("/exchange_rates?quote=USD&base=RUB").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.into_json::<ExchangeRate>().unwrap();
        assert_eq!(usd_rub, body);
    }

    #[test]
    fn get_unauthorized() {
        let client = setup_without_auth();
        let res = client.get("/exchange_rates?quote=EUR&base=USD").dispatch();
        assert_eq!(res.status(), Status::Unauthorized);
    }

    #[test]
    fn get_not_found() {
        let (client, _) = setup();
        let res = client.get("/exchange_rates?quote=EUR&base=USD").dispatch();
        assert_eq!(res.status(), Status::NotFound);
    }

    #[test]
    fn get_sql_query_failed() {
        let (client, db) = setup();
        db.execute_batch("DROP TABLE exchange_rate").unwrap();
        let res = client.get("/exchange_rates?quote=EUR&base=USD").dispatch();
        assert_eq!(res.status(), Status::InternalServerError);
    }
}
