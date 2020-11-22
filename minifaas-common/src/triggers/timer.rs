use chrono::DateTime;
use chrono::Utc;

#[xactor::message(result)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TimerTrigger {
    pub when: DateTime<Utc>,
}

#[xactor::message]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TimerTriggerOutputs {
    exit_code: usize,
}

impl Default for TimerTriggerOutputs {
    fn default() -> Self {
        TimerTriggerOutputs { exit_code: 0 }
    }
}
