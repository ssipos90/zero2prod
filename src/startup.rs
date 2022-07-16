use actix_web::{dev::Server, web, App, HttpServer};
use secrecy::ExposeSecret;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::Settings,
    email_client::EmailClient,
    routes::{health_check, subscribe, confirm},
};

pub struct ApplicationBaseUrl(pub String);

pub fn run(
    listener: TcpListener,
    pool: PgPool,
    email_client: EmailClient,
    base_url: String
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub struct Application {
    pub port: u16,
    pub server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let pool = get_connection_pool(&configuration.database_url.expose_secret());

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");

        let timeout = configuration.email_client.timeout();

        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        let listener = TcpListener::bind(&configuration.application.address).expect(
            format!(
                "Could not bind address {}.",
                &configuration.application.address
            )
            .as_str(),
        );
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            pool,
            email_client,
            configuration.application.base_url
        )?;

        Ok(Self {
            port,
            server
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(connection_url: &str) -> PgPool {
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(connection_url)
        .unwrap()
}
