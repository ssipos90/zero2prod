use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    routes::admin::dashboard::get_username,
    session_state::TypedSession,
    utils::{e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    session: TypedSession,
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = match session.get_user_id().map_err(e500)? {
        Some(user_id) => user_id,
        None => {
            return Ok(see_other("/login"));
        }
    };
    let new_password = form.0.new_password.expose_secret();

    if new_password != form.0.new_password_check.expose_secret() {
        FlashMessage::error("You entered two different passwords!").send();
        return Ok(see_other("/admin/password"));
    }

    if new_password.len() < 12 {
        FlashMessage::error("Your new password should be at least 12 characters in length!").send();
        return Ok(see_other("/admin/password"));
    }

    if new_password.len() > 128 {
        FlashMessage::error("Your new password should be at most 128 characters in length!").send();
        return Ok(see_other("/admin/password"));
    }

    let credentials = Credentials {
        username: get_username(&user_id, &pool).await.map_err(e500)?,
        password: form.0.current_password,
    };

    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("Your current password is incorrect!").send();
                Ok(see_other("/admin/password"))
            }
            AuthError::Unexpected(_) => Err(e500(e)),
        };
    }
    todo!();
}
