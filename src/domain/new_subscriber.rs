use super::{SubscriberEmail, SubscriberName};

#[derive(Debug)]
pub struct NewSubscriber {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
}
