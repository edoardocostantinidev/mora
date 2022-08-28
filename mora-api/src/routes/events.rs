use rocket::Route;

#[get("/")]
fn events() -> &'static str {
    "Healthy"
}

pub fn all() -> Vec<Route> {
    routes![events]
}
