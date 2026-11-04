//! Conversion engine for 釋讀, an experimental Korean orthography that writes
//! Sino-Korean words in Hanja by sound (音讀) and native words in Hanja by
//! meaning (訓讀), with the inflected remainder in Hangul (送韓字).

pub mod conj;
pub mod custom;
pub mod jamo;
pub mod dict;
pub mod dubeolsik;
mod old_jamo;
mod old_spelling;
pub mod engine;
pub mod particles;

pub use engine::{Candidate, Engine, Fragment, Options, Segment, Style};
