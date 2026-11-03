//! 釋讀 (seokdok) input method for Windows: a Text Services Framework text
//! input processor.
//!
//! The key state machine ([`session`]), key vocabulary ([`keys`]), settings
//! ([`config`]) and engine loading ([`engine_host`]) are platform-independent
//! and tested on any host. The TSF glue is compiled on Windows only.

pub mod config;
pub mod engine_host;
pub mod keys;
pub mod session;
