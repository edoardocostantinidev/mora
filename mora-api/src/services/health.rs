use mora_core::result::MoraResult;

#[derive(Debug)]
pub struct HealthCheckResult {
    pub active: bool,
}

pub struct HealthService;

impl HealthService {
    pub fn new() -> Self {
        Self
    }
    pub fn check_system(&self) -> MoraResult<HealthCheckResult> {
        Ok(HealthCheckResult { active: true })
    }
}

#[cfg(test)]
use mockall::predicate::*;

#[cfg(test)]
mockall::mock! {
    pub HealthService {
        pub fn check_system(&self) -> MoraResult<HealthCheckResult>;
    }
}
