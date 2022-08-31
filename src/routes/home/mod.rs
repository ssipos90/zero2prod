use actix_web::{http::header::ContentType, HttpResponse};

pub async fn home() -> HttpResponse {
    let content = include_str!("home.html");
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(content)
}
