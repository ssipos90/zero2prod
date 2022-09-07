use uuid::Uuid;

use crate::helpers::{assert_is_redirect_to, spawn_app};

#[actix_web::test]
async fn you_must_be_logged_in_to_see_the_change_password_form() {
    let app = spawn_app().await;

    let response = app.get_change_password().await;

    assert_is_redirect_to(&response, "/login");
}

#[actix_web::test]
async fn you_must_be_logged_in_to_change_your_password() {
    let app = spawn_app().await;

    let new_password = Uuid::new_v4().to_string();
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &new_password,
        }))
        .await;

    assert_is_redirect_to(&response, "/login");
}

#[actix_web::test]
async fn new_password_fields_must_match() {
    // given
    let app = spawn_app().await;

    app.post_login(&serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password,
    }))
    .await
    .error_for_status()
    .unwrap();

    let new_password = Uuid::new_v4().to_string();
    let new_password_check = Uuid::new_v4().to_string();

    // when
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &new_password_check,
        }))
        .await;

    // then
    assert_is_redirect_to(&response, "/admin/password");

    // when
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("You entered two different passwords!"))
}
