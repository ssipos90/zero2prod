mod dashboard;
mod password;
mod logout;
mod newsletters;

pub use dashboard::admin_dashboard;
pub use password::change_password;
pub use password::change_password_form;
pub use logout::logout;
pub use newsletters::publish_newsletter;
pub use newsletters::publish_newsletter_form;
