use dotenv::from_filename;
use once_cell::sync::Lazy;
use secrecy::{ExposeSecret, Secret};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::{env, io};
use uuid::Uuid;
use wiremock::MockServer;
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

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(
        &self,
        email_request: &wiremock::Request,
    ) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();

            confirmation_link
        };

        ConfirmationLinks {
            html: get_link(body["message"]["html"].as_str().unwrap()),
            plain_text: get_link(body["message"]["text"].as_str().unwrap()),
        }
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let database_name = Uuid::new_v4().to_string();

    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to load configuration.");

        configure_database(&c.database_url, &database_name).await;

        c.database_url = Secret::new(format!(
            "{}/{}",
            c.database_url.expose_secret(),
            &database_name
        ));

        c.email_client.base_url = email_server.uri();

        c
    };

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to bind to address");
    let port = application.port();
    let _ = tokio::spawn(application.server);

    let database_url = configuration.database_url.expose_secret();
    let address_len = configuration.application.address.len();

    let address = format!(
        "http://{}:{}",
        &configuration.application.address[0..address_len - 2],
        port
    );
    TestApp {
        db_pool: get_connection_pool(database_url),
        email_server,
        address,
        port,
    }
}

async fn configure_database(database_url: &Secret<String>, database_name: &str) {
    let database_url = database_url.expose_secret();
    let mut connection = PgConnection::connect(database_url)
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