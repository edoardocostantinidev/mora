use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub timestamp: u128,
    pub queue_name: String,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ScheduleEventRequest {
    data: String,
    schedule_rules: Vec<ScheduleRules>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ScheduleRules {
    schedule_for: u128,
    queue: String,
    recurring_options: Option<RecurringOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RecurringOptions {
    times: u128,
    delay: u128,
}
