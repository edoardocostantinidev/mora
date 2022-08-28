use rocket::serde::Serialize;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Queue {
    name: String,
}

impl From<crate::services::queues::Queue> for Queue {
    fn from(q: crate::services::queues::Queue) -> Self {
        Self { name: q.name }
    }
}
