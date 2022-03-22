extern crate core;

use serde::Serialize;

pub use scenario::Scenario;
pub use schedule::{run_schedule, EventListener};

mod logger;
mod scenario;
mod schedule;

/// A wrapper type for scenario identification.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize)]
pub struct Ident(pub String);
