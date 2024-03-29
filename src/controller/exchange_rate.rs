use crate::{
    model::{ApiResult, ExchangeRate, User},
    service::ExchangeRateService,
};
use rocket::{get, State};

#[get("/exchange_rates?<quote>&<base>")]
pub async fn get(
    quote: &str,
    base: &str,
    service: &State<ExchangeRateService>,
    _user: User,
) -> ApiResult<ExchangeRate> {
    service.get_by_quote_and_base(quote, base).into()
}

#[cfg(test)]
mod test {
    use crate::{model::ExchangeRate, test::client, ExchangeRateRepository};
    use anyhow::Result;
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use rocket::http::Status;

    #[test]
    fn get() -> Result<()> {
        let client = client();
        let repo = client.rocket().state::<ExchangeRateRepository>().unwrap();

        let rate = ExchangeRate {
            quote: "EUR".into(),
            base: "USD".into(),
            rate: 1.25,
        };

        repo.insert_or_replace(&rate)?;

        let res = client.get("/exchange_rates?quote=EUR&base=USD").dispatch();

        assert_eq!(res.status(), Status::Ok);
        let body = res.into_json::<ExchangeRate>().unwrap();
        assert_eq!(rate, body);
        Ok(())
    }

    #[test]
    fn get_inversed() -> Result<()> {
        let client = client();
        let repo = client.rocket().state::<ExchangeRateRepository>().unwrap();

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

        repo.insert_or_replace(&rate)?;

        let res = client.get("/exchange_rates?quote=USD&base=EUR").dispatch();

        assert_eq!(res.status(), Status::Ok);
        let body = res.into_json::<ExchangeRate>().unwrap();
        assert_eq!(inversed_rate, body);
        Ok(())
    }

    #[test]
    fn get_indirect() -> Result<()> {
        let client = client();
        let repo = client.rocket().state::<ExchangeRateRepository>().unwrap();

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

        repo.insert_or_replace(&usd_eur)?;
        repo.insert_or_replace(&rub_eur)?;

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
        let client = client();
        let pool: &Pool<SqliteConnectionManager> = client.rocket().state().unwrap();
        pool.get()
            .unwrap()
            .execute_batch("DELETE FROM auth_token")
            .unwrap();
        let res = client.get("/exchange_rates?quote=EUR&base=USD").dispatch();
        assert_eq!(res.status(), Status::Unauthorized);
    }

    #[test]
    fn get_not_found() {
        let client = client();
        let res = client.get("/exchange_rates?quote=EUR&base=USD").dispatch();
        assert_eq!(res.status(), Status::NotFound);
    }

    #[test]
    fn get_sql_query_failed() {
        let client = client();
        let pool: &Pool<SqliteConnectionManager> = client.rocket().state().unwrap();
        pool.get()
            .unwrap()
            .execute_batch("DROP TABLE exchange_rate")
            .unwrap();
        let res = client.get("/exchange_rates?quote=EUR&base=USD").dispatch();
        assert_eq!(res.status(), Status::InternalServerError);
    }
}
