//! Feature modules. Each one owns its routes, handlers, service rules, repository
//! queries and API shapes. Features never reach into another feature's tables.

pub mod accounts;
pub mod attachments;
pub mod auth;
pub mod calendar;
pub mod entries;
pub mod exports;
pub mod imports;
pub mod insights;
pub mod moods;
pub mod search;
pub mod settings;
pub mod tags;
