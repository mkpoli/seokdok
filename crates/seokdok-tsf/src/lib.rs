//! 釋讀 (seokdok) input method for Windows: a Text Services Framework text
//! input processor.
//!
//! The key state machine, key vocabulary, settings and engine loading live in
//! the shared `seokdok-session` crate (also used by the Linux front end); the
//! TSF glue here is compiled on Windows only.

pub use seokdok_session::{config, engine_host, keys, session};

#[cfg(windows)]
mod candidate_window;
#[cfg(windows)]
mod compartment;
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
pub mod guids;
mod debug_log;
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
