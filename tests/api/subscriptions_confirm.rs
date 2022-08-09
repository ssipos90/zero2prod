use crate::helpers::spawn_app;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[actix_web::test]
async fn confirmations_without_token_are_rejected_with_400() {
    // given
    let app = spawn_app().await;

    // when
   let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();

    // then
    assert_eq!(response.status().as_u16(), 400);
}

#[actix_web::test]
async fn link_returned_by_subscribe_returns_a_200_if_called() {
    // setup
    let app = spawn_app().await;

    // given
    let body = "{\"name\":\"le guin\",\"email\":\"ursula_le_guin@gmail.com\"}";

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    let confirmation_links = app.get_confirmation_links(email_request);

    // when
    let response = reqwest::get(confirmation_links.plain_text)
        .await
        .unwrap();

    // then
    assert_eq!(response.status().as_u16(), 200);
}

#[actix_web::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    // given
    let app = spawn_app().await;
    let body = "{\"name\":\"le guin\",\"email\":\"ursula_le_guin@gmail.com\"}";

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    let confirmation_links = app.get_confirmation_links(email_request);

    // when
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // then
    let saved = sqlx::query_as!(
        ConfirmedSubscriberWithToken,
        r#"SELECT
            s.email, s.name, s.status,
            t.used
        FROM subscriptions AS s
        JOIN subscription_tokens AS t
          ON s.id = t.subscriber_id"#,)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");
    assert!(saved.used);
}

#[actix_web::test]
async fn clicking_again_on_the_confirmation_link_returns_gone() {
    // given
    let app = spawn_app().await;
    let body = "{\"name\":\"le guin\",\"email\":\"ursula_le_guin@gmail.com\"}";

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    let confirmation_links = app.get_confirmation_links(email_request);

    reqwest::get(confirmation_links.html.as_ref())
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let second_response = reqwest::get(confirmation_links.html.as_ref())
        .await
        .unwrap();

    assert_eq!(second_response.status(), 410);
}
