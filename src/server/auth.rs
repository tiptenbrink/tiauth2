use std::sync::Arc;
use axum::extract::Extension;
use axum::Json;
use axum::response::Redirect;
use opaquebind::server::login_server;
use crate::data::key::get_opaque_private;

use crate::data::kv::KeyValue;
use crate::data::source::Source;
use crate::data::user;
use crate::error::Error;
use crate::server::models::{PasswordRequest, PasswordResponse};
use crate::utility::{usp_hex, random_time_hash_hex};

pub async fn start_login(Json(login_start): Json<PasswordRequest>, Extension(dsrc): Extension<Arc<Source>>) -> Result<Json<PasswordResponse>, Error> {
    let private_key = get_opaque_private(&dsrc).await?;

    let user_usph = usp_hex(login_start.username);

    let password_file = user::get_user_by_usph(&dsrc, &user_usph).await?.unwrap_or(
        user::get_user_by_id(&dsrc, 0).await
            .and_then(|u| u.ok_or(Error::RequiredExists))?
    ).password_file;

    let auth_id = random_time_hash_hex(Some(user_usph.into_bytes()));

    let (response, state) = login_server(password_file, login_start.client_request, private_key)?;

    let x = PasswordResponse {
        server_message: response,
        auth_id
    };
    Ok(Json(x))
}