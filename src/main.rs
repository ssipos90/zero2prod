use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use std::{io::stdout, net::TcpListener};
use zero2prod::{
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
    configuration::get_configuration, email_client::EmailClient
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().unwrap();

    let subscriber = get_subscriber(
        "zero2prod".into(),
        "info".into(),
        stdout
    );

    init_subscriber(subscriber);

    let listener = TcpListener::bind(&configuration.application_address)
        .expect(
            format!(
                "Could not bind address {}.",
                &configuration.application_address
            ).as_str()
        );

    let pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(&configuration.database_url.expose_secret())
        .unwrap();

    let sender_email = configuration.email_client.sender()
        .expect("Invalid sender email address.");


    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token
    );

    run(listener, pool, email_client)?.await?;
    Ok(())
}
