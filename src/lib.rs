//! # Kroki-rs
//!
//! `kroki-rs` is a lightweight, high-performance Rust port of the popular [Kroki](https://kroki.io) diagram generation service.
//! It provides a unified API to convert text-based diagram descriptions into images using native CLI tools.

pub mod capabilities;
pub mod cli;
pub mod config;
pub mod diagrams;
pub mod server;
pub mod utils;
