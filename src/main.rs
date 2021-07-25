mod controller;
mod migrations;
mod model;
mod repository;
#[cfg(test)]
mod tests;

#[rocket_sync_db_pools::database("main")]
pub struct Db(rusqlite::Connection);

#[rocket::launch]
fn rocket() -> _ {
    println!("Launching rocket");
    rocket::build()
        .mount("/", rocket::routes![controller::exchange_rates::get])
        .register("/", rocket::catchers![error, error_404])
        .attach(Db::fairing())
        .attach(rocket::fairing::AdHoc::on_ignite(
            "Run migrations",
            run_migrations,
        ))
}

async fn run_migrations(rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
    println!("Running migrations...");
    let db = Db::get_one(&rocket).await.unwrap();
    db.run(|c| migrations::run(c)).await;
    println!("Finished running migrations");

    // NOTE here it returns "no rows" error, which is expected since there are no rows
    db.run(|c| {
        repository::exchange_rates::find_by_base_and_quote(c, "USD".to_string(), "EUR".to_string())
    })
    .await;

    rocket
}

#[rocket::catch(default)]
fn error(
    status: rocket::http::Status,
    req: &rocket::Request,
) -> rocket::serde::json::Json<model::ErrorResponseBody> {
    rocket::serde::json::Json(model::ErrorResponseBody {
        code: status.code,
        message: format!("Failed to handle URI {}", req.uri()),
    })
}

#[rocket::catch(404)]
fn error_404() -> rocket::serde::json::Json<model::ErrorResponseBody> {
    rocket::serde::json::Json(model::ErrorResponseBody {
        code: 404,
        message: "Requested resource does not exist".to_string(),
    })
}
