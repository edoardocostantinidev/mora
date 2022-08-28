use rocket::Route;

#[get("/")]
fn channels() -> &'static str {
    "Healthy"
}

#[get("/<channel_id>", format = "json")]
fn channel(channel_id: String) -> String {
    format!("{channel_id}")
}

#[get("/<channel_id>/events", format = "json")]
fn channel_events(channel_id: String) -> String {
    format!("{channel_id}")
}

#[post("/", format = "json")]
fn create_channel() -> String {
    "".to_owned()
}

#[delete("/<channel_id>", format = "json")]
fn delete_channel(channel_id: String) -> String {
    format!("{channel_id}")
}

#[put("/<channel_id>", format = "json")]
fn edit_channel(channel_id: String) -> String {
    format!("{channel_id}")
}

pub fn all() -> Vec<Route> {
    routes![
        channel,
        channels,
        channel_events,
        create_channel,
        edit_channel,
        delete_channel,
    ]
}
