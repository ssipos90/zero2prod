use anyhow::Context;
use secrecy::ExposeSecret;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    configuration::Settings, domain::SubscriberEmail, email_client::EmailClient,
    startup::get_connection_pool,
};

type PgTransaction = Transaction<'static, Postgres>;

pub enum ExecutionOutcome {
    TaskCompleted,
    EmptyQueue,
}

pub async fn run_worker_until_stopped(configuration: Settings) -> Result<(), anyhow::Error> {
    let pool = get_connection_pool(configuration.database_url.expose_secret());
    let email_client = configuration.email_client.client();
    worker_loop(&pool, email_client).await
}

#[tracing::instrument(
    skip_all,
    fields(
        newsletter_issue_id=tracing::field::Empty,
        subscriber_email=tracing::field::Empty
    ),
    err
)]
pub async fn deliver_queued_tasks(
    pool: &PgPool,
    email_client: &EmailClient,
) -> Result<ExecutionOutcome, anyhow::Error> {
    let mut transaction = pool.begin().await?;

    let (newsletter_issue_id, subscriber_email) = match dequeue_task(&mut transaction)
        .await
        .context("Failed to dequeue newsletter issue.")?
    {
        Some(t) => t,
        None => return Ok(ExecutionOutcome::EmptyQueue),
    };

    match SubscriberEmail::parse(subscriber_email.clone()) {
        Ok(email) => {
            let newsletter_issue = get_issue(pool, &newsletter_issue_id)
                .await
                .context("Failed to fetch issue.")?;
            if let Err(e) = email_client
                .send_email(
                    &email,
                    &newsletter_issue.title,
                    &newsletter_issue.html_content,
                    &newsletter_issue.text_content,
                )
                .await
            {
                tracing::error!(
                    error.cause_chain = ?e,
                    error.message = %e,
                    "Failed to deliver issue to a confirmed subscriber. Skipping.",
                );
            }
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Failed to parse confirmed subscriber email address. Skipping.",
            );
        }
    };
    delete_task(&mut transaction, &newsletter_issue_id, &subscriber_email)
        .await
        .context("waisa")?;
    transaction.commit().await?;

    Ok(ExecutionOutcome::TaskCompleted)
}

async fn worker_loop(pool: &PgPool, email_client: EmailClient) -> Result<(), anyhow::Error> {
    loop {
        match deliver_queued_tasks(pool, &email_client).await {
            Ok(ExecutionOutcome::TaskCompleted) => {}
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await
            }
            Err(_) => tokio::time::sleep(std::time::Duration::from_secs(1)).await,
        };
    }
}

#[derive(FromRow)]
struct NewsletterIssue {
    title: String,
    html_content: String,
    text_content: String,
}

#[tracing::instrument(skip_all)]
async fn get_issue(pool: &PgPool, issue_id: &Uuid) -> Result<NewsletterIssue, sqlx::Error> {
    sqlx::query_as!(
        NewsletterIssue,
        r#"
        SELECT
            title,
            html_content,
            text_content
        FROM newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(pool)
    .await
}

#[tracing::instrument(skip_all)]
async fn delete_task(
    transaction: &mut PgTransaction,
    newsletter_issue_id: &Uuid,
    subscriber_email: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            DELETE FROM issue_delivery_queue
            WHERE
            newsletter_issue_id = $1 AND
            subscriber_email = $2
            "#,
        newsletter_issue_id,
        subscriber_email
    )
    .execute(transaction)
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn dequeue_task(
    transaction: &mut PgTransaction,
) -> Result<Option<(Uuid, String)>, sqlx::Error> {
    sqlx::query!(
        r#"
        SELECT newsletter_issue_id, subscriber_email
        FROM issue_delivery_queue
        FOR UPDATE
        SKIP LOCKED
        LIMIT 1
        "#
    )
    .fetch_optional(transaction)
    .await
    .map(|maybe_result| {
        maybe_result.map(|result| (result.newsletter_issue_id, result.subscriber_email))
    })
}
