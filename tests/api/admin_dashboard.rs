use crate::helpers::{assert_is_redirect_to, spawn_app};

#[actix_web::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    let app = spawn_app().await;

    let response = app.get_admin_dashboard().await;

    assert_is_redirect_to(&response, "/login");
}

#[actix_web::test]
async fn logout_clears_session() {
    let app = spawn_app().await;
    app.login_test_user().await.unwrap();

    let response = app
        .post_login(&serde_json::json!({
            "username": app.test_user.username,
            "password": app.test_user.password,
        }))
        .await
        .error_for_status()
        .unwrap();

    // then
    assert_is_redirect_to(&response, "/admin/dashboard");

    // when
    let html_page = app.get_admin_dashboard_html().await;
    // then we see welcome page
    assert!(html_page.contains(&format!(
        "<p>Welcome, {}!</p>",
        app.test_user.username
    )));

    // when
    let response = app.post_logout().await;
    // then
    assert_is_redirect_to(&response, "/login");

    // when
    let html_page = app.get_login_html().await;
    // then
    assert!(html_page.contains("<p><i>You have successfully logged out.</i></p>"));

    // when
    let response = app.get_admin_dashboard().await;
    // then
    assert_is_redirect_to(&response, "/login");
}
