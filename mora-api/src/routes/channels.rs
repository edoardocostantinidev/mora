use axum::{extract::State, http::StatusCode, Json};
use log::{debug, error};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Deserialize)]
pub struct CreateChannelRequest {
    queue_filter: String,
}

#[derive(Serialize)]
pub struct CreateChannelResponse {
    channel_id: String,
}

pub async fn create_channel(
    State(app_state): State<AppState>,
    request: Json<CreateChannelRequest>,
) -> Result<Json<CreateChannelResponse>, (StatusCode, String)> {
    debug!("Received request for channel creation...");
    let queue_filter = request.queue_filter.parse().map_err(|e| {
        error!("invalid queue_filter, make sure it's a regex! {e}");
        (
            StatusCode::BAD_REQUEST,
            format!("invalid queue_filter, make sure it's a regex! {e}"),
        )
    })?;
    debug!("valid filter {:?}", &queue_filter);
    let channel = app_state
        .channel_manager
        .lock()
        .await
        .create_channel(queue_filter)
        .map_err(|e| {
            error!("couldn't create channel: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("couldn't create channel: {e}"),
            )
        })?;
    debug!("channel created {:?}", &channel);
    Ok(Json(CreateChannelResponse {
        channel_id: channel.id().to_owned(),
    }))
}

pub async fn list_channels(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    app_state
        .channel_manager
        .lock()
        .await
        .get_channels()
        .map(|channels| Json(channels.into_iter().map(|c| c.id().to_owned()).collect()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
