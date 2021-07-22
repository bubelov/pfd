#[cfg(test)]
mod tests;
mod migrations;
mod model;
mod controller;
mod repository;

#[rocket_sync_db_pools::database("main")]
pub struct Db(rusqlite::Connection);

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", rocket::routes![controller::exchange_rates::get])
        .register("/", rocket::catchers![error, error_404])
        .attach(Db::fairing())
        .attach(rocket::fairing::AdHoc::on_ignite("Run migrations", run_migrations))
}

async fn run_migrations(rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
    let db = Db::get_one(&rocket).await.unwrap();
    db.run(|c| migrations::run(c)).await;
    rocket
}

#[rocket::catch(default)]
fn error(status: rocket::http::Status, req: &rocket::Request) -> rocket::serde::json::Json<model::ErrorResponseBody> {
    rocket::serde::json::Json(
        model::ErrorResponseBody {
            code: status.code,
            message: format!("Failed to handle URI {}", req.uri())
        }
    )
}

#[rocket::catch(404)]
fn error_404() -> rocket::serde::json::Json<model::ErrorResponseBody> {
    rocket::serde::json::Json(
        model::ErrorResponseBody {
            code: 404,
            message: "Requested resource does not exist".to_string()
        }
    )
}
