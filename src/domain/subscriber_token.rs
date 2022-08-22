use rand::{distributions::Alphanumeric, thread_rng, Rng};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriptionToken(String);

const TOKEN_SIZE: usize = 25;

impl SubscriptionToken {
    pub fn parse(s: String) -> Result<SubscriptionToken, String> {
        if s.trim().is_empty() {
            return Err("Token cannot be empty".to_string());
        }

        if s.graphemes(true).count() != TOKEN_SIZE {
            return Err(format!("String token must be of {} length", TOKEN_SIZE));
        }

        if s.chars().any(|c| !c.is_alphanumeric()) {
            return Err("Token contains forbidden characters".to_string());
        }

        Ok(SubscriptionToken(s))
    }

    pub fn generate() -> SubscriptionToken {
        let mut rng = thread_rng();

        SubscriptionToken(
            std::iter::repeat_with(|| rng.sample(Alphanumeric))
                .map(char::from)
                .take(TOKEN_SIZE)
                .collect(),
        )
    }
}

impl AsRef<str> for SubscriptionToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};
    use unicode_segmentation::UnicodeSegmentation;

    use super::{SubscriptionToken, TOKEN_SIZE};

    #[test]
    fn generated_token_is_of_proper_length() {
        assert_eq!(
            SubscriptionToken::generate()
                .as_ref()
                .graphemes(true)
                .count(),
            TOKEN_SIZE
        );
    }

    #[test]
    fn generated_token_is_alphanum() {
        assert!(SubscriptionToken::generate()
            .as_ref()
            .chars()
            .all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn parsing_token_fails_if_too_long() {
        assert_err!(SubscriptionToken::parse(
            "abcdefghijklmnopqrstuvwxyz".to_string()
        ));
    }

    #[test]
    fn parsing_token_fails_if_too_short() {
        assert_err!(SubscriptionToken::parse(
            "abcdefghijklm".to_string()
        ));
    }

    #[test]
    fn parsing_token_fails_if_non_alphanum() {
        assert_err!(SubscriptionToken::parse(
            "abcdefghijklabcdefghijkl!".to_string()
        ));
    }

    #[test]
    fn parsing_token_succeeds_with_alnum() {
        assert_ok!(SubscriptionToken::parse(
            "abcdefghijklabcdefghijkl2".to_string()
        ));
    }
}
