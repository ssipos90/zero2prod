use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

#[derive(Debug)]
pub struct EmailClient {
    pub sender: SubscriberEmail,
    pub http_client: Client,
    pub base_url: String,
    pub authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            http_client: Client::builder().timeout(timeout).build().unwrap(),
            base_url,
            authorization_token,
            sender,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/send", self.base_url);
        let recepient = SendEmailMessageRecepient {
            email: recipient.as_ref(),
            name: "Test",
        };

        let request_body = SendEmailMessageRequest {
            key: &self.authorization_token.expose_secret(),
            message: SendEmailMessageDetails {
                to: [recepient],
                from_email: self.sender.as_ref(),
                from_name: self.sender.as_ref(),
                subject,
                html: html_content,
                text: text_content,
            },
        };

        let _builder = self
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
struct SendEmailMessageRecepient<'a> {
    email: &'a str,
    name: &'a str,
}

#[derive(serde::Serialize)]
struct SendEmailMessageDetails<'a> {
    from_email: &'a str,
    from_name: &'a str,
    to: [SendEmailMessageRecepient<'a>; 1],
    subject: &'a str,
    html: &'a str,
    text: &'a str,
}

#[derive(serde::Serialize)]
struct SendEmailMessageRequest<'a> {
    key: &'a str,
    message: SendEmailMessageDetails<'a>,
}

#[cfg(test)]
mod tests {
    use claim::assert_err;
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use wiremock::{
        matchers::{any, header, method, path},
        Mock, MockServer, Request, ResponseTemplate,
    };

    use super::*;

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                match body.get("message") {
                    Some(message) => {
                        message.get("from_email").is_some()
                            && message.get("from_name").is_some()
                            && message.get("to").is_some()
                            && message.get("subject").is_some()
                            && message.get("html").is_some()
                            && message.get("text").is_some()
                            && message.get("to").is_some()
                    }
                    None => false,
                }
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        // Given
        let mock_server = MockServer::start().await;
        let email_client = fake_email_client(mock_server.uri());

        Mock::given(method("POST"))
            .and(header("Content-Type", "application/json"))
            .and(path("/send"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1) // Then
            .mount(&mock_server)
            .await;

        // When
        let _ = email_client
            .send_email(
                fake_email(),
                &fake_subject(),
                &fake_content(),
                &fake_content(),
            )
            .await;
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // Given
        let mock_server = MockServer::start().await;
        let email_client = fake_email_client(mock_server.uri());
        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));

        Mock::given(any())
            .respond_with(response)
            .expect(1) // When
            .mount(&mock_server)
            .await;
        let outcome = email_client
            .send_email(
                fake_email(),
                &fake_subject(),
                &fake_content(),
                &fake_content(),
            )
            .await;

        // Then
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // Given
        let mock_server = MockServer::start().await;
        let email_client = fake_email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1) // When
            .mount(&mock_server)
            .await;
        let outcome = email_client
            .send_email(
                fake_email(),
                &fake_subject(),
                &fake_content(),
                &fake_content(),
            )
            .await;

        // Then
        assert_err!(outcome);
    }

    fn fake_subject() -> String {
        Sentence(1..2).fake()
    }

    fn fake_content() -> String {
        Paragraph(1..10).fake()
    }

    fn fake_email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn fake_email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            fake_email(),
            Secret::new(Faker.fake()),
            std::time::Duration::from_millis(200),
        )
    }
}
