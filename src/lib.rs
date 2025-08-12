//! Public API facade for the `kern` crate.

pub mod cipher;
pub mod reduction;
pub mod ciphers;
pub mod core;

pub use ciphers::calculate_all;
