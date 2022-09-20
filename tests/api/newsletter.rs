use std::time::Duration;

use fake::{
    faker::{internet::en::SafeEmail, name::en::Name},
    Fake,
};
use wiremock::{
    matchers::{any, method, path},
    Mock, MockBuilder, ResponseTemplate,
};

use crate::helpers::{assert_is_redirect_to, spawn_app, TestApp};

fn when_sending_an_email() -> MockBuilder {
    Mock::given(path("/email")).and(method("POST"))
}

#[actix_web::test]
async fn transient_errors_do_not_cause_duplicate_deliveries_on_retries() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    create_confirmed_subscriber(&app).await;
    app.login_test_user().await.unwrap();

    let newsletter_request_body = serde_json::json!({
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text.",
        "html_content": "<p>Newsletter body as HTML.</p>",
    });
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app.post_newsletter(&newsletter_request_body).await;
    assert_eq!(response.status().as_u16(), 500);

    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .named("Delivery retry")
        .mount(&app.email_server)
        .await;
    let response = app.post_newsletter(&newsletter_request_body).await;
    assert_eq!(response.status().as_u16(), 303);

    // Mock tests for **not** sending duplicates
}

#[actix_web::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.login_test_user().await.unwrap();

    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // first
    let newsletter_request_body = serde_json::json!({
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text.",
        "html_content": "<p>Newsletter body as HTML.</p>",
    });

    let (response1, response2) = tokio::join!(
        app.post_newsletter(&newsletter_request_body),
        app.post_newsletter(&newsletter_request_body),
    );
    assert_eq!(response1.status(), response2.status());
    assert_eq!(
        response1.text().await.unwrap(),
        response2.text().await.unwrap()
    );

    // Mock verifies we sent newsletter only *once*
}

#[actix_web::test]
async fn creation_is_idempotent() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.login_test_user().await.unwrap();

    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // first
    let newsletter_request_body = serde_json::json!({
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text.",
        "html_content": "<p>Newsletter body as HTML.</p>",
    });

    let response = app.post_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_newsletter_html().await;
    assert!(html_page.contains("The newsletter issue has been published!"));

    // second
    let response = app.post_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html_page = app.get_newsletter_html().await;
    assert!(html_page.contains("The newsletter issue has been published!"));
}

#[actix_web::test]
async fn returns_400_for_invalid_data() {
    let app = spawn_app().await;
    app.login_test_user().await.unwrap();

    let test_cases = vec![
        (
            serde_json::json!({
                "idempotency_key": uuid::Uuid::new_v4().to_string(),
                "text_content": "Newsletter body as plain text.",
                "html_content": "<p>Newsletter body as HTML.</p>",
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "idempotency_key": uuid::Uuid::new_v4().to_string(),
                "title": "Newsletter title",
                "html_content": "<p>Newsletter body as HTML.</p>",
            }),
            "missing text content",
        ),
        (
            serde_json::json!({
                "idempotency_key": uuid::Uuid::new_v4().to_string(),
                "title": "Newsletter title",
                "text_content": "Newsletter body as plain text.",
            }),
            "missing html content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletter(&invalid_body).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with '400 Bad Request' when the payload was '{}.'",
            error_message
        );
    }
}

#[actix_web::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    app.login_test_user().await.unwrap();

    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text.",
        "html_content": "<p>Newsletter body as HTML.</p>",
    });

    let response = app.post_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
}

#[actix_web::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    app.login_test_user().await.unwrap();

    create_confirmed_subscriber(&app).await;

    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text.",
        "html_content": "<p>Newsletter body as HTML.</p>",
    });

    let response = app.post_newsletter(&newsletter_request_body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> crate::helpers::ConfirmationLinks {
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let body = serde_urlencoded::to_string(&serde_json::json!({
        "name": name,
        "email": email,
    }))
    .unwrap();

    dbg!("body: ", &body);

    let _mock_guard = when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body)
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_links = create_unconfirmed_subscriber(app).await;

    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}
