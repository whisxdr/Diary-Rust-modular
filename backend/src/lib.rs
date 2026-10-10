//! Dear My Diary backend. A modular monolith: HTTP concerns on the outside,
//! business rules in feature modules, SQL in feature repositories.

pub mod app;
pub mod config;
pub mod features;
pub mod infrastructure;
pub mod middleware;
pub mod shared;
pub mod state;
