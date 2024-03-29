use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

use crate::utils::html_messages;

pub async fn change_password_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let msg_html = html_messages(&flash_messages);

    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(
        format!(r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8"/>
        <title>Change password</title>
    </head>
    <body>
        {msg_html}
        <form method="post" action="/admin/password">
            <label>Current password
                <input type="text" placeholder="Enter current password" name="current_password" />
            </label>
            <label>New password
                <input type="password" placeholder="Enter new password" name="new_password" />
            </label>
            <label>Confirm new password
                <input type="password" placeholder="Enter new password again" name="new_password_check" />
            </label>

            <button type="submit">Change password</button>
        </form>
        <p>
          <a href="/admin/dashboard">&lt;- Back</a>
        </p>
    </body>
</html>"#,
    )))
}
