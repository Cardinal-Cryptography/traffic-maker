use serde::Deserialize;

pub use logs::Logs;
pub use scenario::{Scenario, Status};

mod logs;
mod scenario;

#[derive(Clone, Debug, Deserialize)]
pub struct Ident(pub String);
