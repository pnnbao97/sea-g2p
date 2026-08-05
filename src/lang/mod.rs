//! One module per language, each owning its own normalizer tables, rules and
//! script-specific machinery. Anything shared lives in [`crate::core`].
//!
//! - [`vi`] — Vietnamese: the staged text normalizer (numbers, dates, units,
//!   abbreviations) plus the syllable knowledge the G2P engine leans on.
//! - [`en`] — English: the frequency wordlist that settles ambiguous splits
//!   and language decisions in [`crate::g2p`].
//! - [`th`] — Thai: word segmentation (the script has no spaces), Unicode
//!   spelling normalization, and dictionary-backed pronunciation.

pub mod en;
pub mod th;
pub mod vi;
