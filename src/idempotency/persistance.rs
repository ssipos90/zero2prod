use actix_web::{body::to_bytes, HttpResponse};
use anyhow::Context;
use reqwest::StatusCode;
use sqlx::{postgres::PgHasArrayType, FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::IdempotencyKey;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPair {
    name: String,
    value: Vec<u8>,
}

impl PgHasArrayType for HeaderPair {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_header_pair")
    }
}

#[derive(FromRow)]
struct SavedResponse {
    response_status_code: i16,
    response_headers: Vec<HeaderPair>,
    response_body: Vec<u8>,
}

#[allow(clippy::large_enum_variant)]
pub enum NextAction {
    StartProcessing(Transaction<'static, Postgres>),
    ReturnSavedResponse(HttpResponse),
}

pub async fn try_processing(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: &Uuid,
) -> Result<NextAction, anyhow::Error> {
    let mut transaction = pool.begin().await?;
    sqlx::query!("SET TRANSACTION ISOLATION LEVEL repeatable read")
        .execute(&mut transaction)
        .await?;
    let n_inserted_rows = sqlx::query!(
        r#"
        INSERT INTO idempotency (
            user_id,
            idempotency_key,
            created_at
        )
        VALUES ($1, $2, now())
        ON CONFLICT DO NOTHING
        "#,
        user_id,
        idempotency_key.as_ref()
    )
    .execute(&mut transaction)
    .await?
    .rows_affected();
    if n_inserted_rows > 0 {
        Ok(NextAction::StartProcessing(transaction))
    } else {
        let saved_response = get_saved_response(pool, idempotency_key, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to find a saved response."))?;
        Ok(NextAction::ReturnSavedResponse(saved_response))
    }
}

pub async fn get_saved_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: &Uuid,
) -> Result<Option<HttpResponse>, anyhow::Error> {
    let maybe_result = sqlx::query_as!(
        SavedResponse,
        r#"
        SELECT
          response_status_code as "response_status_code!",
          response_headers as "response_headers!: Vec<HeaderPair>",
          response_body as "response_body!"
        FROM idempotency
        WHERE
          user_id = $1 AND
          idempotency_key = $2
        "#,
        user_id,
        idempotency_key.as_ref()
    )
    .fetch_optional(pool)
    .await
    .context("Failed to fetch idempotency key.")?;

    if let Some(result) = maybe_result {
        let status_code = StatusCode::from_u16(result.response_status_code.try_into()?)?;
        let mut response = HttpResponse::build(status_code);
        for HeaderPair { name, value } in result.response_headers {
            response.append_header((name, value));
        }
        Ok(Some(response.body(result.response_body)))
    } else {
        Ok(None)
    }
}

pub async fn save_response<'a>(
    mut transaction: Transaction<'a, Postgres>,
    idempotency_key: &IdempotencyKey,
    user_id: &Uuid,
    http_response: HttpResponse,
) -> Result<HttpResponse, anyhow::Error> {
    let (response_head, body) = http_response.into_parts();
    let status_code = response_head.status().as_u16() as i16;
    let body = to_bytes(body).await.map_err(|e| anyhow::anyhow!("{}", e))?;
    let headers: Vec<_> = response_head
        .headers()
        .iter()
        .map(|(name, value)| HeaderPair {
            name: name.as_str().to_owned(),
            value: value.as_bytes().to_owned(),
        })
        .collect();

    sqlx::query_unchecked!(
        r#"
        UPDATE idempotency
        SET
            response_status_code = $3,
            response_headers = $4,
            response_body = $5
        WHERE
            user_id = $1 AND
            idempotency_key = $2
        "#,
        user_id,
        idempotency_key.as_ref(),
        status_code,
        headers,
        body.as_ref()
    )
    .execute(&mut transaction)
    .await?;
    transaction.commit().await?;

    Ok(response_head.set_body(body).map_into_boxed_body())
}
