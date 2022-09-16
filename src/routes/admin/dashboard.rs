use actix_web::{error::ErrorInternalServerError, http::header::ContentType, web, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{session_state::TypedSession, utils::{e500, see_other}};

pub async fn admin_dashboard(
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> actix_web::Result<HttpResponse> {
    let username = if let Some(user_id) = session.get_user_id().map_err(ErrorInternalServerError)? {
        get_username(&user_id, &pool)
            .await
            .map_err(e500)?
    } else {
        return Ok(see_other("/login"));
    };
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8"/>
        <title>Admin dashboard</title>
    </head>
    <body>
        <p>Welcome, {username}!</p>
        <ol>
          <li><a href="/admin/password">Change password</a></li>
        </ol>
    </body>
</html>
            "#
        )))
}

pub async fn get_username(user_id: &Uuid, pool: &PgPool) -> anyhow::Result<String> {
    let row = sqlx::query!("SELECT * FROM users WHERE user_id=$1", user_id)
        .fetch_one(pool)
        .await
        .context("Failed to perform a query to fetch an username.")?;

    Ok(row.username)
}
