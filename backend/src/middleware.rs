use std::any::Any;

use salvo::{handler, prelude::StatusError, Depot, Request, Response};

use crate::{global, model, service};

#[handler]
pub async fn with_admin(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let token_data = jwt_data_from_request(req, res)?;
    let user =
        model::user::select_by_steamid(&mut global::RB.clone(), token_data.steamid64.clone())
            .await
            .map_err(|_| StatusError::unauthorized())?;
    match user {
        Some(user) => {
            if user.is_admin {
                Ok(())
            } else {
                Err(StatusError::unauthorized())
            }
        }
        None => Err(StatusError::unauthorized()),
    }
}

#[handler]
pub fn with_jwt(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<(), StatusError> {
    let token_data = jwt_data_from_request(req, res)?;
    set_request_depot(depot, "token_data", token_data);
    Ok(())
}

fn set_request_depot<T: Any + Send>(depot: &mut Depot, key: &str, data: T) {
    depot.insert(key, data);
}

fn jwt_data_from_request(
    req: &mut Request,
    res: &mut Response,
) -> Result<service::auth::TokenData, StatusError> {
    let jwt_token = req
        .headers()
        .get("Authorization")
        .map(|v| v.to_str().unwrap().replace("Bearer ", ""))
        .ok_or(StatusError::unauthorized())?;
    match service::auth::decode_token(&jwt_token) {
        Ok(token_data) => Ok(token_data),
        Err(e) => {
            res.render(e.to_string());
            Err(StatusError::unauthorized())
        }
    }
}
