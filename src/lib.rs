#![allow(dead_code)]

mod data;
mod error;
mod utility;
mod auth;
mod server;
mod config;

pub use crate::server::run_server;
