use actix_http::header::LOCATION;
use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;

pub fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}

pub fn html_errors(flash_messages: IncomingFlashMessages) -> String {
    flash_messages
        .iter()
        .fold(String::new(), |a, m| a + &format!("<p><i>{}</i></p>", m.content()))
}
