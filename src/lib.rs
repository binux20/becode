//! 🐝 BeCode - Autonomous AI coding agent library
//!
//! This library provides the core functionality for the BeCode agent,
//! including tools, providers, sessions, and the TUI interface.

pub mod agent;
pub mod attachments;
pub mod config;
pub mod permissions;
pub mod providers;
pub mod session;
pub mod tools;
pub mod tui;
pub mod utils;

pub use config::Config;
pub use permissions::Permission;
