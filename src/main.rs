use dotenv::{dotenv, var};
use sqlx::PgPool;
use std::{io::stdout, net::TcpListener};
use zero2prod::{startup::run, telemetry::{get_subscriber, init_subscriber}};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let subscriber = get_subscriber("zero2prod".into(), "info".into(), stdout);

    init_subscriber(subscriber);

    let database_url = var("DATABASE_URL").expect("No DATABASE_URL env var");
    let port = var("PORT").map_or(8000, |v| v.parse().expect("PORT cannot be parsed as i32"));
    let address = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&address)
        .expect(format!("Could not bind address {}.", &address).as_str());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres.");

    run(listener, pool)?.await
}
