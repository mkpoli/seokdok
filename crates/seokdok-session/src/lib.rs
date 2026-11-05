//! The platform-independent core of the 釋讀 input methods, shared by the
//! Windows (TSF) and Linux (fcitx5) front ends: the key state machine
//! ([`session`]), the key vocabulary ([`keys`]), the settings file
//! ([`config`]), the Windows quick menu ([`quick_menu`]), the rotation of the script
//! ([`script_cycle`]), the choice of where
//! the composition is drawn ([`surface`]), where the popup goes on the screen ([`placement`]), the engine loader with the reader's customisation ([`engine_host`]),
//! what a person keeps picking ([`learned`]) and the short readings of a phone's candidate bar ([`brief`]).

pub mod brief;
pub mod config;
pub mod engine_host;
pub mod keys;
pub mod learned;
pub mod mode_keys;
pub mod placement;
pub mod quick_menu;
pub mod script_cycle;
pub mod session;
pub mod surface;
