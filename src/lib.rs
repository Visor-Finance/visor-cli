//! The `visor` command-line interface for the Visor personal API.
//!
//! The binary in `main.rs` only parses arguments and maps errors to exit
//! codes; everything else lives here so tests can reach it.

pub mod api;
pub mod banner;
pub mod cli;
pub mod client;
pub mod commands;
pub mod config;
pub mod credentials;
pub mod error;
pub mod input;
pub mod models;
pub mod oauth;
pub mod output;
pub mod resolve;
