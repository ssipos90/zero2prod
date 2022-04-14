use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;
use chrono::Utc;

use crate::domain::{SubscriberName, NewSubscriber};

#[derive(serde::Deserialize)]
pub struct SubscribeForm {
    email: String,
    name: String,
}

#[tracing::instrument(
    skip(form, pool),
    fields(
        subscriber_name = %form.name,
        subscriber_email = %form.email
    )
)]
pub async fn subscribe(form: web::Json<SubscribeForm>, pool: web::Data<PgPool>) -> impl Responder {
    let subscriber = NewSubscriber {
        name: SubscriberName::parse(form.0.name),
        email: form.0.email,
    };
    match insert_subscriber(&pool, &subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

#[tracing::instrument(skip(form, pool))]
pub async fn insert_subscriber(pool: &PgPool, form: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
