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
    ) -> Self {
        Self {
            http_client: Client::new(),
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
            .await?;

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
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use wiremock::{
        matchers::{header, header_exists, method, path},
        Mock, MockServer, Request, ResponseTemplate,
    };

    use super::*;

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                dbg!(&body);
                match body.get("message") {
                    Some(message) => {
                        message.get("from_name").is_some()
                            && message.get("to").is_some()
                            && message.get("subject").is_some()
                            && message.get("html_body").is_some()
                            && message.get("text_body").is_some()
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
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(Faker.fake()));
        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        Mock::given(header("Content-Type", "application/json"))
            .and(path("/send"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1) // Then
            .mount(&mock_server)
            .await;

        // When
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;
    }
}
