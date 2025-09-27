use std::{convert::Infallible, net::SocketAddr};

use axum::{
    extract::{ConnectInfo, FromRef},
    http::request::Parts,
    Error, RequestPartsExt,
};

use crate::AppState;

pub struct ConnectionMiddleware;

const MORA_ID_KEY_HEADER: &str = "MORA-ID-KEY";

impl<S> axum::extract::FromRequestParts<S> for ConnectionMiddleware
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = &'static str;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let connect_info = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .ok_or("Missing ConnectInfo")?;

        let Some(id) = parts
            .headers
            .get(MORA_ID_KEY_HEADER)
            .and_then(|value| std::str::from_utf8(value.as_bytes()).ok())
        else {
            return Err("Missing MORA-ID-KEY header");
        };

        app_state
            .connections
            .lock()
            .await
            .add_client(id.to_string(), connect_info.to_string());
        Ok(Self)
    }
}
