use secrecy::Secret;
use dotenv::{dotenv, var, Error};

pub struct Settings {
    pub database_url: Secret<String>,
    pub application_address: String,
}

pub fn get_configuration() -> Result<Settings, Error> {
    dotenv().ok();

    Ok(Settings {
        database_url: Secret::new(var("DATABASE_URL").expect("DATABASE_URL missing")),
        application_address: format!(
            "{}:{}",
            var("HTTP_INTERFACE").map_or("127.0.0.1".to_string(), |x| x),
            var("HTTP_PORT").map_or(8000, |v| v.parse::<u16>().expect("PORT cannot be parsed as u16"))
        )
    })
}

