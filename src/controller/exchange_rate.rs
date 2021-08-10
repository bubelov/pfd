use crate::{
    db::Db,
    model::{ApiResult, ExchangeRate, User},
    service::exchange_rate,
};
use rocket::get;

#[get("/exchange_rates?<quote>&<base>")]
pub async fn get(quote: &str, base: &str, db: Db, _user: User) -> ApiResult<ExchangeRate> {
    exchange_rate::get_by_quote_and_base(quote, base, db)
        .await
        .into()
}

#[cfg(test)]
mod test {
    use crate::{
        model::ExchangeRate,
        repository::exchange_rate,
        test::{setup, setup_without_auth},
    };
    use anyhow::Result;
    use rocket::http::Status;

    #[test]
    fn get() -> Result<()> {
        let (client, mut db) = setup();

        let rate = ExchangeRate {
            quote: "EUR".into(),
            base: "USD".into(),
            rate: 1.25,
        };

        exchange_rate::insert_or_replace(&rate, &mut db)?;

        let res = client.get("/exchange_rates?quote=EUR&base=USD").dispatch();

        assert_eq!(res.status(), Status::Ok);
        let body = res.into_json::<ExchangeRate>().unwrap();
        assert_eq!(rate, body);
        Ok(())
    }

    #[test]
    fn get_inversed() -> Result<()> {
        let (client, mut db) = setup();

        let rate = ExchangeRate {
            quote: "EUR".into(),
            base: "USD".into(),
            rate: 1.19,
        };

        let inversed_rate = ExchangeRate {
            quote: "USD".into(),
            base: "EUR".into(),
            rate: 1.0 / 1.19,
        };

        exchange_rate::insert_or_replace(&rate, &mut db)?;

        let res = client.get("/exchange_rates?quote=USD&base=EUR").dispatch();

        assert_eq!(res.status(), Status::Ok);
        let body = res.into_json::<ExchangeRate>().unwrap();
        assert_eq!(inversed_rate, body);
        Ok(())
    }

    #[test]
    fn get_indirect() -> Result<()> {
        let (client, mut db) = setup();

        let usd_eur = ExchangeRate {
            quote: "USD".into(),
            base: "EUR".into(),
            rate: 0.840972163821378,
        };

        let rub_eur = ExchangeRate {
            quote: "RUB".into(),
            base: "EUR".into(),
            rate: 0.0115324823898994,
        };

        exchange_rate::insert_or_replace(&usd_eur, &mut db)?;
        exchange_rate::insert_or_replace(&rub_eur, &mut db)?;

        let rub_usd = ExchangeRate {
            quote: "RUB".into(),
            base: "USD".into(),
            rate: 0.0115324823898994 / 0.840972163821378,
        };

        let res = client.get("/exchange_rates?quote=RUB&base=USD").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.into_json::<ExchangeRate>().unwrap();
        assert_eq!(rub_usd, body);

        let usd_rub = ExchangeRate {
            quote: "USD".into(),
            base: "RUB".into(),
            rate: 0.840972163821378 / 0.0115324823898994,
        };

        let res = client.get("/exchange_rates?quote=USD&base=RUB").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.into_json::<ExchangeRate>().unwrap();
        assert_eq!(usd_rub, body);
        Ok(())
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
