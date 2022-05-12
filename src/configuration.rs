use secrecy::Secret;
use dotenv::{dotenv, var, Error};

use crate::domain::SubscriberEmail;

pub struct EmailClientSettings {
    pub base_url: String,
    pub authorization_token: Secret<String>,
    pub sender_email: String,
}

pub struct Settings {
    pub database_url: Secret<String>,
    pub application_address: String,
    pub email_client: EmailClientSettings,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
}

pub fn get_configuration() -> Result<Settings, Error> {
    dotenv().ok();

    Ok(Settings {
        database_url: Secret::new(var("DATABASE_URL").expect("DATABASE_URL missing")),
        application_address: format!(
            "{}:{}",
            var("HTTP_INTERFACE").map_or("127.0.0.1".to_string(), |x| x),
            var("HTTP_PORT").map_or(8000, |v| v.parse::<u16>().expect("PORT cannot be parsed as u16"))
        ),
        email_client: EmailClientSettings {
            authorization_token: Secret::new(var("EMAIL_CLIENT_AUTHORIZATION_TOKEN").expect("EMAIL_CLIENT_AUTHORIZATION_TOKEN missing")),
            base_url: var("EMAIL_CLIENT_BASE_URL").expect("EMAIL_CLIENT_BASE_URL missing"),
            sender_email: var("EMAIL_CLIENT_SENDER_EMAIL").expect("EMAIL_CLIENT_SENDER_EMAIL missing"),
        },
    })
}

