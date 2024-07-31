//! ConfigCat OpenFeature Provider for Rust.

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::module_name_repetitions)]

/// ConfigCat provider module.
mod provider;
pub use provider::*;

pub use configcat;
pub use open_feature;
