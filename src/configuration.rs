use dotenv::{dotenv, var, Error};
use secrecy::Secret;

use crate::domain::SubscriberEmail;

#[derive(Clone, Debug)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub authorization_token: Secret<String>,
    pub sender_email: String,
    pub timeout_milliseconds: u64,
}

#[derive(Clone, Debug)]
pub struct ApplicationSettings {
    pub address: String,
    pub base_url: String,
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub database_url: Secret<String>,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

pub fn get_configuration() -> Result<Settings, Error> {
    dotenv().ok();

    Ok(Settings {
        database_url: Secret::new(var("DATABASE_URL").expect("DATABASE_URL missing")),
        application: ApplicationSettings {
            address: format!(
                "{}:{}",
                var("HTTP_INTERFACE").unwrap_or_else(|_| "127.0.0.1".to_string()),
                var("HTTP_PORT").map_or(8000, |v| v
                    .parse::<u16>()
                    .expect("PORT cannot be parsed as u16"))
            ),
            base_url: var("BASE_URL").unwrap_or_else(|_| "http://127.0.0.1".to_string())
        },
        email_client: EmailClientSettings {
            authorization_token: Secret::new(
                var("EMAIL_CLIENT_AUTHORIZATION_TOKEN")
                    .expect("EMAIL_CLIENT_AUTHORIZATION_TOKEN missing"),
            ),
            base_url: var("EMAIL_CLIENT_BASE_URL").expect("EMAIL_CLIENT_BASE_URL missing"),
            sender_email: var("EMAIL_CLIENT_SENDER_EMAIL")
                .expect("EMAIL_CLIENT_SENDER_EMAIL missing"),
            timeout_milliseconds: var("EMAIL_CLIENT_TIMEOUT_MILLISECONDS").map_or(5000, |v| {
                v.parse::<u64>()
                    .expect("EMAIL_CLIENT_TIMEOUT_MILLISECONDS cannot be parsed as u64")
            }),
        },
    })
}
