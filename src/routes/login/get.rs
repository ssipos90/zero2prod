use actix_web::{http::header::ContentType, HttpResponse};

pub async fn login_form() -> HttpResponse {
    let error_html = "".to_string();

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
        {error_html}
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
