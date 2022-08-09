use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use tracing::{instrument, log::error};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Params {
    subscription_token: String,
}

#[instrument(name = "Confirm a pending subscriber", skip(parameters))]
pub async fn confirm(pool: web::Data<PgPool>, parameters: web::Query<Params>) -> HttpResponse {
    match get_token_info(&pool, &parameters.subscription_token).await {
        Ok(None) => HttpResponse::Unauthorized().finish(),
        Ok(Some((subscriber_id, used))) => {
            if used {
                HttpResponse::Gone().finish()
            } else {
                if confirm_subscriber(&pool, subscriber_id).await.is_err() {
                    return HttpResponse::InternalServerError().finish();
                }
                HttpResponse::Ok().finish()
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[instrument(skip(pool, subscriber_id))]
async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    // TODO: delete token?
    sqlx::query!(
        "UPDATE subscriptions SET status='confirmed' WHERE id = $1",
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[instrument(skip(pool, subscription_token))]
async fn get_token_info(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<(Uuid, bool)>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT subscriber_id, used FROM subscription_tokens WHERE subscription_token=$1",
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(result.map(|r| (r.subscriber_id, r.used)))
}
