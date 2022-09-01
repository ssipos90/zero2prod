use crate::helpers::{spawn_app, assert_is_redirect_to};

#[actix_web::test]
async fn an_error_flash_message_is_set_on_failure() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password",
    });

    let response = app.post_login(&login_body).await;

    // then the response should redirect us to the login page
    assert_is_redirect_to(&response, "/login");

    // when loading the login form
    let html_page = app.get_login_html().await;
    // then we see the error message
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));

    // when we reload the login form
    let html_page = app.get_login_html().await;
    // then the error message should dissapear
    assert!(!html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
}
