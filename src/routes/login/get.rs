use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

use crate::utils::html_messages;

#[tracing::instrument(skip(flash_messages))]
pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
     let msg_html = html_messages(&flash_messages);

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8"/>
        <title>Login</title>
    </head>
    <body>
        {msg_html}
        <form method="post" action="/login">
            <label>Username
                <input type="text" placeholder="Enter username" name="username" />
            </label>
            <label>Password
                <input type="password" placeholder="Enter password" name="password" />
            </label>

            <button type="submit">Login</button>
        </form>
    </body>
</html>"#
        ))
}
