use rocket::{get, Route};

#[get("/")]
fn health() -> &'static str {
    "Healthy"
}

pub fn all() -> Vec<Route> {
    routes![health]
}
