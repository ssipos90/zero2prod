use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    domain::SubscriberEmail,
    email_client::EmailClient,
    routes::error_chain_fmt,
};
use actix_http::{
    header::{self, HeaderMap, HeaderValue},
    StatusCode,
};
use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use anyhow::Context;
use secrecy::Secret;
use sqlx::PgPool;

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication failed.")]
    Auth(#[source] anyhow::Error),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Auth(_) => StatusCode::FORBIDDEN,
            Self::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse<actix_http::body::BoxBody> {
        match self {
            Self::Unexpected(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
            Self::Auth(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
                response
                    .headers_mut()
                    .insert(header::WWW_AUTHENTICATE, header_value);

                response
            }
        }
    }
}

#[tracing::instrument(
    skip(body, pool, email_client, request),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    request: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let credentials = basic_authentication(request.headers()).map_err(PublishError::Auth)?;
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => PublishError::Auth(e.into()),
            AuthError::Unexpected(_) => PublishError::Unexpected(e.into()),
        })?;
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

    let subscribers = get_confirmed_subscribers(&pool).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}.", subscriber.email)
                    })?;
            }
            Err(error) => {
                tracing::warn!(
                    // record the error chain as a structured field
                    // on the log record.
                    error.cause_chain = ?error,
                    // backslash doesn't create a \n, it's just for readability
                    "Skipping a confirmed subscriber. \
                     Their stored contact details are invalid",
                );
            }
        }
    }
    Ok(HttpResponse::Ok().finish())
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing.")?
        .to_str()
        .context("The 'Authorization' header was not valid UTF8.")?;
    let encoded = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;

    let decoded_bytes = base64::decode_config(encoded, base64::STANDARD)
        .context("Failed to base64-decode 'Basic' credentials.")?;

    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8.")?;

    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("An username must be provided in 'Basic' auth."))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[tracing::instrument(skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let subscribers = sqlx::query!("SELECT email FROM subscriptions WHERE status = 'confirmed'")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect();

    Ok(subscribers)
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}