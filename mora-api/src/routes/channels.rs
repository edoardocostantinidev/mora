use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use log::{debug, error, info};
use mora_channel::Channel;
use mora_core::{
    clock::Clock,
    models::channels::{
        BufferOptions, CreateChannelRequest, CreateChannelResponse, Event,
        GetChannelEventsResponse, GetChannelResponse, ListChannelsResponse,
    },
};

/// Creates a channel.
pub async fn create_channel(
    State(app_state): State<AppState>,
    request: Json<CreateChannelRequest>,
) -> Result<Json<CreateChannelResponse>, (StatusCode, String)> {
    debug!("Received request for channel creation");
    let channel = app_state
        .channel_manager
        .lock()
        .await
        .create_channel(
            request.queues.clone(),
            request.buffer_options.size,
            request.buffer_options.time,
        )
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

/// List active channels
pub async fn list_channels(
    State(app_state): State<AppState>,
) -> Result<Json<ListChannelsResponse>, (StatusCode, String)> {
    let channels = app_state
        .channel_manager
        .lock()
        .await
        .get_channels()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .into_iter()
        .map(|channel| mora_core::models::channels::Channel {
            channel_id: channel.id().to_owned(),
            queues: channel.queues().to_owned(),
            buffer_options: BufferOptions {
                size: channel.buffer_size(),
                time: channel.buffer_time(),
            },
            msec_from_last_op: channel.msec_from_last_op(),
        })
        .collect();
    Ok(Json(ListChannelsResponse { channels }))
}

/// Get an active channel
pub async fn get_channel(
    State(app_state): State<AppState>,
    channel_id: Path<String>,
) -> Result<Json<GetChannelResponse>, (StatusCode, String)> {
    let mut channel_manager = app_state.channel_manager.lock().await;
    let channel = channel_manager
        .get_mut_channel(&channel_id.0)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match channel {
        None => Err((
            StatusCode::NOT_FOUND,
            format!("{} channel does not exist", &channel_id.0),
        )),
        Some(channel) => Ok(Json(GetChannelResponse {
            channel_id: channel.id().to_owned(),
            queues: channel.queues().to_owned(),
            buffer_options: BufferOptions {
                size: channel.buffer_size(),
                time: channel.buffer_time(),
            },
            msec_from_last_op: channel.msec_from_last_op(),
        })),
    }
}

pub async fn get_channel_events(
    State(app_state): State<AppState>,
    channel_id: Path<String>,
) -> Result<Json<GetChannelEventsResponse>, (StatusCode, String)> {
    info!("Received get_channel_events request");
    let mut channel_manager = app_state.channel_manager.lock().await;
    let mut queue_pool = app_state.queue_pool.lock().await;
    let channel_opt = channel_manager
        .get_mut_channel(&channel_id.0)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    match channel_opt {
        Some(channel) => {
            channel.reset_msec_from_last_op();
            let timestamp = Clock::now();
            let delta = channel.buffer_time();
            let mut events: Vec<Event> = vec![];
            let queues = channel.queues();
            info!("Found {:?}", &queues);
            for queue_name in queues {
                let queue = queue_pool
                    .get_queue_mut(queue_name)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                info!("Queue Found {:?}", &queue);
                let data = queue.dequeue_until(timestamp + delta);
                info!("Data Found {:?}", &data);
                let dequeued_events: Result<Vec<_>, _> = data
                    .iter()
                    .map(|data| {
                        Ok(Event {
                            data: std::str::from_utf8(data)
                                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                                .to_owned(),
                        })
                    })
                    .collect();
                events.extend(dequeued_events?)
            }

            Ok(Json(GetChannelEventsResponse { events }))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            format!("{} channel does not exist", &channel_id.0),
        )),
    }
}

/// Deletes an active channel
pub async fn delete_channel(State(app_state): State<AppState>, channel_id: Path<String>) {
    let mut channel_manager = app_state.channel_manager.lock().await;
    channel_manager.close_channel(&channel_id.0)
}
