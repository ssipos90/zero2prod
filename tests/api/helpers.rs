use dotenv::from_filename;
use once_cell::sync::Lazy;
use secrecy::{ExposeSecret, Secret};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::{env, io};
use uuid::Uuid;
use zero2prod::{
    configuration::get_configuration,
    startup::{get_connection_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    from_filename(".env.testing").ok();
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let database_name = Uuid::new_v4().to_string();
    let configuration = {
        let mut c = get_configuration().expect("Failed to load configuration.");

        configure_database(&c.database_url, &database_name).await;

        c.database_url = Secret::new(format!(
            "{}{}",
            c.database_url.expose_secret(),
            &database_name
        ));

        c
    };

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to bind to address");
    let port = application.port();
    let _ = tokio::spawn(application.server);

    let database_url = configuration.database_url.expose_secret();
    let address_len = configuration.application_address.len();

    let address = format!(
        "http://{}:{}",
        &configuration.application_address[0..address_len - 2],
        port
    );
    TestApp {
        db_pool: get_connection_pool(database_url),
        address
    }
}

async fn configure_database(database_url: &Secret<String>, database_name: &str) {
    let database_url = database_url.expose_secret();
    let mut connection = PgConnection::connect(&database_url)
        .await
        .expect("Failed to connect to DB");

    connection
        .execute(sqlx::query(&format!(
            r#"CREATE DATABASE "{}";"#,
            &database_name
        )))
        .await
        .expect("Failed to create the DB.");

    let db_pool = PgPool::connect(format!("{}/{}", &database_url, &database_name).as_str())
        .await
        .expect("Failed to connect to DB");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate DB.");
}
