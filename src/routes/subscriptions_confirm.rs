use actix_web::{web, HttpResponse};
use tracing::instrument;

#[derive(serde::Deserialize)]
pub struct Params {
    subscription_token: String,
}

#[instrument(name = "Confirm a pending subscriber", skip(_parameters))]
pub async fn confirm(_parameters: web::Query<Params>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
