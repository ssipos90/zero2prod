use dotenv::{var,Error};

pub fn get_configuration() -> Result<Settings, Error> {
    let port_str = var("PORT").unwrap();
    Ok(Settings {
        database_url: var("DATABASE_URL").unwrap(),
        application_port: port_str.parse::<u16>().unwrap(),
    })
}

pub struct Settings {
    pub database_url: String,
    pub application_port: u16
}

