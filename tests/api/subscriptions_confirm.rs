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
