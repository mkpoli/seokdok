//! The platform-independent core of the 釋讀 input methods, shared by the
//! Windows (TSF) and Linux (fcitx5) front ends: the key state machine
//! ([`session`]), the key vocabulary ([`keys`]), the settings file
//! ([`config`]) and the engine loader with the reader's customisation
//! ([`engine_host`]).

pub mod config;
pub mod engine_host;
pub mod keys;
pub mod session;
