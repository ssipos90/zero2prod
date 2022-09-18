use actix_web::{http::header::ContentType, web, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{authentication::UserId, utils::e500};

pub async fn admin_dashboard(
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> actix_web::Result<HttpResponse> {
    let username = get_username(&user_id, &pool).await.map_err(e500)?;
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
          <li>
            <form action="/admin/logout" name="logoutForm" method="post">
              <button name="logout" value="" type="submit">Logout</button>
            </form>
          </li>
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
