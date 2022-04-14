use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub name: SubscriberName,
    pub email: String,
}

pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> SubscriberName {
        if s.trim().is_empty() {
            panic!("Subscriber name cannot be empty")
        }

        if s.graphemes(true).count() > 256 {
            panic!("Subscriber name is too long")
        }

        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        if s.chars().any(|g| forbidden_chars.contains(&g)) {
            panic!("Subscriber name contains forbidden characters")
        }

        SubscriberName(s)
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
