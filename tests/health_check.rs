use std::{io, env, net::TcpListener};
use reqwest::Client;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{startup::run, telemetry::{get_subscriber, init_subscriber}};
use dotenv::{dotenv,var};
use once_cell::sync::Lazy;

static TRACING: Lazy<()> = Lazy::new(|| {
    dotenv().ok();
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, io::sink);
        init_subscriber(subscriber);
    }
});

#[actix_web::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = Client::new();

    let response = client
        .get(&format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_web::test]
async fn subscriber_returns_a_200_for_valid_form_data() {
    // setup
    let app = spawn_app().await;

    let client = Client::new();

    // given
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // when
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
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
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // when
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // then
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fial with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

struct TestApp {
    address: String,
    db_pool: PgPool,
}

async fn spawn_app () -> TestApp {
    Lazy::force(&TRACING);

    let database_url = var("TEST_DATABASE_URL").expect("No DATABASE_URL env var");
    let database_name = Uuid::new_v4().to_string();
    let db_pool = prepare_db(&database_url, &database_name).await;
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port(); 
    let server = run(listener, db_pool.clone()).expect("Failed to bind to address");
    let _ = tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool
    }
}

async fn prepare_db (database_url: &str, database_name: &str) -> PgPool {
    let mut connection = PgConnection::connect(&database_url)
        .await
        .expect("Failed to connect to DB");

    connection
        .execute(sqlx::query(&format!(r#"CREATE DATABASE "{}";"#, &database_name)))
        .await
        .expect("Failed to create the DB.");

    let db_pool = PgPool::connect(format!("{}/{}", &database_url, &database_name).as_str())
        .await
        .expect("Failed to connect to DB");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate DB.");
    
    db_pool
}
