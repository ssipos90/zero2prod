use reqwest::Client;
use crate::helpers::spawn_app;

#[actix_web::test]
async fn subscriber_returns_a_200_for_valid_form_data() {
    // setup
    let app = spawn_app().await;

    let client = Client::new();

    // given
    let body = "{\"name\":\"le guin\",\"email\":\"ursula_le_guin@gmail.com\"}";

    // when
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // then

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[actix_web::test]
async fn subscriber_returns_a_400_when_data_is_missing() {
    // setup
    let app = spawn_app().await;
    let client = Client::new();

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
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/json")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        let status = response.status().as_u16();
        let body = response.text().await.unwrap();

        // then
        assert_eq!(
            400, status,
            "The API did not fial with 400 Bad Request when the payload was {}.",
            error_message
        );
        assert!(body.contains(error_message));
    }
}

#[actix_web::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    // setup
    let app = spawn_app().await;
    let client = Client::new();

    // given
    let test_cases = vec![
        ("{\"name\":\"\",\"email\":\"ursula_le_guin@gmail.com\"}", "empty field `email`"),
        ("{\"name\":\"Ursula\",\"email\":\"\"}", "empty field `email`"),
        ("{\"name\":\"Ursula\",\"email\":\"not an@-email\"}", "invalid `email`"),
    ];
    for (body, description) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");
        let status = response.status().as_u16();
        // let body = response.text().await.unwrap();

        // Assert
        assert_eq!(
            400,
            status,
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
