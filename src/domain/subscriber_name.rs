use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        if s.trim().is_empty() {
            return Err("Subscriber name cannot be empty".to_string());
        }

        if s.graphemes(true).count() > 256 {
            return Err("Subscriber name is too long".to_string());
        }

        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        if s.chars().any(|g| forbidden_chars.contains(&g)) {
            return Err("Subscriber name contains forbidden characters".to_string());
        }

        Ok(SubscriberName(s))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
  use crate::domain::SubscriberName;
  use claim::{assert_err, assert_ok};

  #[test]
  fn a_256_grapheme_subscriber_name_is_ok() {
    let name = "a".repeat(256);
    assert_ok!(SubscriberName::parse(name.to_string()));
  }

  #[test]
  fn a_name_longer_than_256_grapheme_is_an_error() {
    let name = "a".repeat(257);
    assert_err!(SubscriberName::parse(name.to_string()), "Subscriber name is too long");
  }

  #[test]
  fn whitespace_only_subscriber_name_is_an_error() {
    assert_err!(SubscriberName::parse(" ".to_string()), "Subscriber name cannot be empty");
  }

  #[test]
  fn empty_subscriber_name_is_an_error() {
    let name = "".to_string();
    assert_err!(SubscriberName::parse(name), "Subscriber name cannot be empty");
  }

  #[test]
  fn name_contain_invalid_characters_is_an_error() {
      for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name), "Subscriber name contains forbidden characters");
      }
  }

  #[test]
  fn a_valid_subscriber_name_is_ok() {
    let name = "Miguel".to_string();
    assert_ok!(SubscriberName::parse(name));
  }
}
