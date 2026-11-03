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

#[cfg(windows)]
mod candidate_window;
#[cfg(windows)]
mod composition;
#[cfg(windows)]
mod configure;
#[cfg(windows)]
pub mod dll;
#[cfg(windows)]
mod display_attribute;
#[cfg(windows)]
mod edit_session;
#[cfg(windows)]
mod factory;
#[cfg(windows)]
mod guids;
#[cfg(windows)]
mod key_event_sink;
#[cfg(windows)]
mod lang_bar;
#[cfg(windows)]
mod registry;
#[cfg(windows)]
mod settings_dialog;
#[cfg(windows)]
mod text_input_processor;
#[cfg(windows)]
mod text_service;
#[cfg(windows)]
mod thread_mgr_event_sink;

#[cfg(windows)]
pub(crate) use dll::{dll_instance, lock_module, unlock_module};
