use std::sync::Arc;
use axum::extract::Extension;
use axum::Json;
use opaquebind::server::{login_server, login_server_finish, register_server, register_server_finish};

use crate::data::key::{get_opaque_private, get_opaque_public};
use crate::data::kv::KeyValue;
use crate::data::source::Source;
use crate::data::user;
use crate::data::user::{new_user_return_id, User};
use crate::error::{Error};
use crate::error::BadFlow::ExpiredAuthId;
use crate::server::models::{FinishLogin, FinishRegister, FlowUser, PasswordRequest, PasswordResponse, SavedState};
use crate::utility;
use crate::utility::{usp_hex};

pub async fn start_login(Json(login_start): Json<PasswordRequest>, Extension(dsrc): Extension<Arc<Source>>) -> Result<Json<PasswordResponse>, Error> {
    let private_key = get_opaque_private(&dsrc).await?;

    let user_usph = usp_hex(&login_start.username);

    let password_file = user::get_user_by_usph(&dsrc, &user_usph).await?.unwrap_or(
        // id 0 is the fake record
        user::get_user_by_id(&dsrc, 0).await?.ok_or(Error::RequiredNotExists)?
    ).password_file;

    let auth_id = utility::random_time_hash_hex(Some(user_usph.as_bytes()));

    let (response, state) = login_server(password_file, login_start.client_request, private_key)?;
    let saved_state = SavedState {
        user_usph,
        state
    };

    dsrc.kv.store_json(&auth_id, &saved_state, 60).await?;

    let x = PasswordResponse {
        server_message: response,
        auth_id
    };
    Ok(Json(x))
}

pub async fn finish_login(Json(login_finish): Json<FinishLogin>, Extension(dsrc): Extension<Arc<Source>>) -> Result<(), Error> {
    let saved_state: SavedState = dsrc.kv.get_json(&login_finish.auth_id).await?
        .ok_or(Error::BadFlow(ExpiredAuthId))?;
    let user_usph = usp_hex(&login_finish.username);
    if user_usph != saved_state.user_usph {
      return Err(Error::IncorrectFinishUsername)
    };
    let session_key = login_server_finish(login_finish.client_request, saved_state.state)?;
    let auth_time = utility::utc_timestamp();
    let flow_user = FlowUser {
        flow_id: login_finish.flow_id,
        user_usph,
        auth_time
    };

    dsrc.kv.store_json(&session_key, &flow_user, 60).await?;

    Ok(())
}

pub async fn start_register(Json(register_start): Json<PasswordRequest>, Extension(dsrc): Extension<Arc<Source>>) -> Result<Json<PasswordResponse>, Error> {
    let public_key = get_opaque_public(&dsrc).await?;

    let user_usph = usp_hex(&register_start.username);

    let auth_id = utility::random_time_hash_hex(Some(user_usph.as_bytes()));

    let (response, state) = register_server(register_start.client_request, public_key)?;
    let saved_state = SavedState {
        user_usph,
        state
    };

    let _: () = dsrc.kv.store_json(&auth_id, &saved_state, 60).await?;

    let x = PasswordResponse {
        server_message: response,
        auth_id
    };
    Ok(Json(x))
}

pub async fn finish_register(Json(login_finish): Json<FinishRegister>, Extension(dsrc): Extension<Arc<Source>>) -> Result<(), Error> {
    let saved_state: SavedState = dsrc.kv.get_json(&login_finish.auth_id).await?
        .ok_or(Error::BadFlow(ExpiredAuthId))?;
    let user_usph = usp_hex(&login_finish.username);
    if user_usph != saved_state.user_usph {
        return Err(Error::IncorrectFinishUsername)
    };

    let password_file = register_server_finish(login_finish.client_request, saved_state.state)?;

    let new_user = User {
        usp_hex: user_usph,
        id: 0,
        password_file
    };

    let _ = new_user_return_id(&dsrc, &new_user).await?;

    Ok(())
}