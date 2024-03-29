use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, dev::Server, web, App, HttpServer};
use actix_web_flash_messages::{storage::CookieMessageStore, FlashMessagesFramework};
use actix_web_lab::middleware::from_fn;
use anyhow::Context;
use secrecy::{ExposeSecret, Secret};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::Settings,
    email_client::EmailClient,
    routes::{
        admin_dashboard, change_password, change_password_form, confirm, health_check, home, login,
        login_form, logout, publish_newsletter, subscribe, publish_newsletter_form,
    }, authentication::reject_anonymous_users,
};

pub struct ApplicationBaseUrl(pub String);

#[tracing::instrument(skip(listener,pool,email_client,hmac_secret,redis_uri))]
pub async fn run(
    listener: TcpListener,
    pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: Secret<String>,
    redis_uri: Secret<String>,
) -> anyhow::Result<Server> {
    let db_pool = web::Data::new(pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));

    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();

    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await.context("Cannot connect to redis.")?;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(message_framework.clone())
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .wrap(TracingLogger::default())
            .route("/", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .service(
                web::scope("/admin")
                    .wrap(from_fn(reject_anonymous_users))
                    .route("/dashboard", web::get().to(admin_dashboard))
                    .route("/logout", web::post().to(logout))
                    .route("/password", web::get().to(change_password_form))
                    .route("/password", web::post().to(change_password))
                    .route("/newsletters", web::get().to(publish_newsletter_form))
                    .route("/newsletters", web::post().to(publish_newsletter))
            )
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)
    .context("Cannot start HTTP server.")?
    .run();

    Ok(server)
}

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);

pub struct Application {
    pub port: u16,
    pub server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> anyhow::Result<Self> {
        let pool = get_connection_pool(configuration.database_url.expose_secret());

        let email_client = configuration.email_client.client();

        let listener = TcpListener::bind(&configuration.application.address).unwrap_or_else(|_| {
            panic!(
                "Could not bind address {}.",
                &configuration.application.address
            )
        });
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            pool,
            email_client,
            configuration.application.base_url,
            configuration.application.hmac_secret,
            configuration.redis_uri,
        )
        .await?;

        Ok(Self { port, server })
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
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(connection_url)
        .unwrap()
}
