use crate::helpers::spawn_app;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[actix_web::test]
async fn subscriber_returns_a_200_for_valid_form_data() {
    // setup
    let app = spawn_app().await;

    // given
    let body = "{\"name\":\"le guin\",\"email\":\"ursula_le_guin@gmail.com\"}";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // when
    let response = app.post_subscriptions(body.into()).await;

    // then
    assert_eq!(200, response.status().as_u16());
}

#[actix_web::test]
async fn subscribe_persists_the_new_subscriber() {
    // setup
    let app = spawn_app().await;

    // given
    let body = "{\"name\":\"le guin\",\"email\":\"ursula_le_guin@gmail.com\"}";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // when
    app.post_subscriptions(body.into()).await;

    // then
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation");
}

#[actix_web::test]
async fn subscriber_returns_a_400_when_data_is_missing() {
    // setup
    let app = spawn_app().await;

    // given
    let test_cases = vec![
        ("{\"name\":\"le guin\"}", "missing field `email`"),
        (
            "{\"email\":\"ursula_le_guin@gmail.com\"}",
            "missing field `name`",
        ),
        ("{}", "missing field"),
    ];

    for (invalid_body, error_message) in test_cases {
        // when
        let response = app.post_subscriptions(invalid_body.into()).await;
        let status = response.status().as_u16();
        let body = response.text().await.unwrap();

        // then
        assert_eq!(
            400, status,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
        assert!(body.contains(error_message));
    }
}

#[actix_web::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    // setup
    let app = spawn_app().await;

    // given
    let test_cases = vec![
        (
            "{\"name\":\"\",\"email\":\"ursula_le_guin@gmail.com\"}",
            "empty field `email`",
        ),
        (
            "{\"name\":\"Ursula\",\"email\":\"\"}",
            "empty field `email`",
        ),
        (
            "{\"name\":\"Ursula\",\"email\":\"not an@-email\"}",
            "invalid `email`",
        ),
    ];
    for (body, description) in test_cases {
        // Act
        let response = app.post_subscriptions(body.into()).await;
        let status = response.status().as_u16();
        // let body = response.text().await.unwrap();

        // Assert
        assert_eq!(
            400, status,
            "The API did not return a 400 Bad Request when the payload was '{}'.",
            description
        );
        // assert!(
        //     body.contains(description),
        //     "Body does not contain \"{}\"",
        //     description
        // );
    }
}

#[actix_web::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    // given
    let body = "{\"name\":\"le guin\",\"email\":\"ursula_le_guin@gmail.com\"}";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
}

#[actix_web::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = spawn_app().await;
    // given
    let body = "{\"name\":\"le guin\",\"email\":\"ursula_le_guin@gmail.com\"}";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // when
    app.post_subscriptions(body.into()).await;

    // then
    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    let confirmation_links = app.get_confirmation_links(email_request);

    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}

#[actix_web::test]
async fn re_subscribe_sends_same_confirmation_email() {
    let app = spawn_app().await;
    // given
    let body = "{\"name\":\"le guin\",\"email\":\"ursula_le_guin@gmail.com\"}";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let first_response = app.post_subscriptions(body.into()).await;

    assert_eq!(200, first_response.status().as_u16());

    let first_email_request = &app.email_server.received_requests().await.unwrap()[0];
    let first_confirmation_links = app.get_confirmation_links(first_email_request);

    app.email_server.reset().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let second_response = app.post_subscriptions(body.into()).await;

    assert_eq!(200, second_response.status().as_u16());

    let second_email_request = &app.email_server.received_requests().await.unwrap()[0];
    let second_confirmation_links = app.get_confirmation_links(second_email_request);

    assert_eq!(first_confirmation_links.plain_text, second_confirmation_links.plain_text);
}

#[actix_web::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    let app = spawn_app().await;

    // given
    let body = "{\"name\":\"le guin\",\"email\":\"ursula_le_guin@gmail.com\"}";

    sqlx::query!("ALTER TABLE subscriptions DROP COLUMN email;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(response.status().as_u16(), 500);
}
