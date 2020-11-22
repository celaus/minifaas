use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod http;
pub mod timer;
use http::HttpMethod;

///
/// Represents a trigger for the Function as a Service function. Declares the required parameters and so on. Defaults to `None` which means disabled.
///
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
#[serde(tag = "type", content = "when")]
pub enum Trigger {
    /// Execute on a specified HTTP call
    Http(HttpMethod),

    Interval(Duration),

    /// Disable a function
    None,
}

impl Default for Trigger {
    fn default() -> Self {
        Trigger::Http(HttpMethod::default())
    }
}

impl Trigger {
    ///
    /// Creates a list of all available HTTP triggers.
    ///
    pub fn all_http() -> Vec<Trigger> {
        HttpMethod::available()
            .into_iter()
            .map(Trigger::Http)
            .collect()
    }

    pub fn is_http(&self) -> bool {
        if let Trigger::Http(_) = *self {
            true
        } else {
            false
        }
    }

    pub fn is_timer(&self) -> bool {
        if let Trigger::Interval(_) = *self {
            true
        } else {
            false
        }
    }
    pub fn is_disabled(&self) -> bool {
        if let Trigger::None = *self {
            true
        } else {
            false
        }
    }
}

impl std::fmt::Display for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            Trigger::Http(method) => format!("HTTP ({:?})", method),
            Trigger::Interval(pause) => format!("Interval (every {:?})", pause),
            Trigger::None => "Disabled".to_owned(),
        };
        write!(f, "{}", text)
    }
}

pub trait RawOutputConver {}
