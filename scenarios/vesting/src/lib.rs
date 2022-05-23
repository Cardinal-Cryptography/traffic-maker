// Needed for `do_async!`.
#![feature(fn_traits)]

pub use schedules_merging::{SchedulesMerging, SchedulesMergingConfig};

mod events;
mod schedules_merging;
