//! English-specific data. The G2P engine itself is language-agnostic and
//! lives in [`crate::g2p`]; what belongs here is the evidence used to decide
//! *whether a token is English*.

pub mod top_words;
