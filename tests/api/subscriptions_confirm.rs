use crate::helpers::spawn_app;
use reqwest::Url;
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
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let raw_confirmation_link = &get_link(&body["message"]["html"].as_str().unwrap());
    let confirmation_link = Url::parse(raw_confirmation_link).unwrap();
    // Make sure we are not calling some random APIs on the web
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    // when
    let response = reqwest::get(confirmation_link)
        .await
        .unwrap();

    // then
    assert_eq!(response.status().as_u16(), 200);
}
