//! Building blocks for cipher implementations.
//! 
//! The functions in this module operate on plain characters and numbers
//! without any knowledge about higher level cipher strategies.

pub mod letters;
pub mod numbers;

pub use letters::*;
pub use numbers::*;
