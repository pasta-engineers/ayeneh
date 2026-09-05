//! Mirror Benchmark core library.
//!
//! Contains all shared application/domain functionality including benchmarking,
//! mirror loading, pip installation, reporting, and scheduling.

pub mod app;
pub mod benchmark;
pub mod config;
pub mod mirror;
pub mod npm;
pub mod pip;
pub mod report;
pub mod scheduler;
