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
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);

        let request_body = SendEmailMessage {
            to: [SendEmailAddress{
                email: recipient.as_ref(),
                name: "Test",
            }],
            sender: SendEmailAddress {
              email: self.sender.as_ref(),
              name: self.sender.as_ref(),
            },
            subject,
            html_content,
            text_content,
        };

        let _builder = self
            .http_client
            .post(&url)
            .header("api-key", self.authorization_token.expose_secret())
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
struct SendEmailAddress<'a> {
    email: &'a str,
    name: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SendEmailMessage<'a> {
    sender: SendEmailAddress<'a>,
    to: [SendEmailAddress<'a>; 1],
    subject: &'a str,
    html_content: &'a str,
    text_content: &'a str,
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
                body.get("sender").is_some()
                    && body.get("to").is_some()
                    && body.get("subject").is_some()
                    && body.get("htmlContent").is_some()
                    && body.get("textContent").is_some()
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
            .and(path("/email"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1) // Then
            .mount(&mock_server)
            .await;

        // When
        let _ = email_client
            .send_email(
                &fake_email(),
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
        let response = ResponseTemplate::new(200)
            // 3 minutes
            .set_delay(std::time::Duration::from_secs(180));

        Mock::given(any())
            .respond_with(response)
            .expect(1) // When
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(
                &fake_email(),
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
                &fake_email(),
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
