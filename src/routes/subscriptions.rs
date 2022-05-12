use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use tracing::{error,instrument};
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberName, SubscriberEmail};

#[derive(serde::Deserialize)]
pub struct SubscribeForm {
    email: String,
    name: String,
}

impl TryFrom<SubscribeForm> for NewSubscriber {
    type Error = String;

    fn try_from(form: SubscribeForm) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;

        Ok(NewSubscriber {
            name,
            email,
        })
    }
}

#[instrument(
    skip(form, pool),
    fields(
        subscriber_name = %form.name,
        subscriber_email = %form.email
    )
)]
pub async fn subscribe(form: web::Json<SubscribeForm>, pool: web::Data<PgPool>) -> impl Responder {
    let subscriber = match form.into_inner().try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&pool, &subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[instrument(skip(form, pool))]
pub async fn insert_subscriber(pool: &PgPool, form: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email.as_ref(),
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
