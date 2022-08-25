use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid email", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claim::{assert_err,assert_ok};
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_email_parsers(email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(email.0).is_ok()
    }

    #[test]
    fn empty_string_is_not_valid() {
        assert_err!(SubscriberEmail::parse("".to_string()));
    }

    #[test]
    fn email_missing_at_sign_is_not_valid() {
        assert_err!(SubscriberEmail::parse("sometest.com".to_string()));
    }

    #[test]
    fn email_missing_subject_is_not_valid() {
        assert_err!(SubscriberEmail::parse("@test.com".to_string()));
    }

    #[test]
    fn valid_emails_pass() {
        let email = SafeEmail().fake();
        assert_ok!(SubscriberEmail::parse(email));
    }
}
