#![allow(dead_code)]

mod data;
mod error;
mod utility;
mod auth;
mod server;

pub use crate::server::run_server;
