use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

use crate::utils::html_messages;

pub async fn publish_newsletter_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let msg_html = html_messages(&flash_messages);
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8"/>
        <title>Publish a newsletter</title>
    </head>
    <body>
        {msg_html}
        <form method="post" action="/admin/publish">
            <label>Subject
                <input type="text" placeholder="Enter email subject" name="subject" />
            </label>
            <label>Body
                <textarea type="text" placeholder="Enter email content" name="content" />
            </label>

            <button type="submit">Publish</button>
        </form>
    </body>
</html>"#
        ))
}
