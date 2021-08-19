use crate::{
    model::{ApiResult, ExchangeRate, User},
    repository::ExchangeRateRepository,
    service::exchange_rate,
};
use rocket::{get, State};

#[get("/exchange_rates?<quote>&<base>")]
pub async fn get(
    quote: &str,
    base: &str,
    repo: &State<ExchangeRateRepository>,
    _user: User,
) -> ApiResult<ExchangeRate> {
    exchange_rate::get_by_quote_and_base(quote, base, repo).into()
}

#[cfg(test)]
mod test {
    use crate::{
        model::ExchangeRate,
        test::{setup, setup_without_auth},
        ExchangeRateRepository,
    };
    use anyhow::Result;
    use rocket::http::Status;

    #[test]
    fn get() -> Result<()> {
        let (client, _) = setup();
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
        let (client, _) = setup();
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
        let (client, _) = setup();
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
        let (client, _) = setup_without_auth();
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
