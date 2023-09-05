use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct Clock;

impl Clock {
    pub fn now() -> u128 {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Impossible");
        since_the_epoch.as_nanos()
    }
}
