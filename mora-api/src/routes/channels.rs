use axum::{extract::State, http::StatusCode, Json};
use log::{debug, error};
use mora_channel::ChannelManager;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateChannelRequest {
    queue_filter: String,
}

#[derive(Serialize)]
pub struct CreateChannelResponse {
    channel_id: String,
}

pub async fn create_channel(
    State(mut channel_manager): State<ChannelManager>,
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
    debug!("{:?}", channel_manager);
    let channel = channel_manager.create_channel(queue_filter).map_err(|e| {
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
    State(channel_manager): State<ChannelManager>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    channel_manager
        .get_channels()
        .map(|channels| Json(channels.into_iter().map(|c| c.id().to_owned()).collect()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
