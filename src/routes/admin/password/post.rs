use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::{
    authentication::{validate_credentials, AuthError, Credentials, UserId},
    routes::admin::dashboard::get_username,
    utils::{e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>
) -> Result<HttpResponse, actix_web::Error> {
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

    let current_password = form.0.current_password;

    let credentials = Credentials {
        username: get_username(&user_id, &pool).await.map_err(e500)?,
        password: current_password.clone(),
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
    crate::authentication::change_password(&user_id, &form.0.new_password, &pool)
        .await
        .map_err(e500)?;

    FlashMessage::info("Your password has been changed!").send();

    Ok(see_other("/admin/password"))
}
