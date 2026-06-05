//! This crate provides a basic implementation of the Unicode Collation Algorithm. There is really
//! just one method, `collate`, belonging to a struct, `Collator`, which sets a few options. Despite
//! the bare-bones API, this implementation conforms to the standard and allows for the use of the
//! CLDR root collation order; so it may indeed be useful.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::too_long_first_doc_paragraph)]

mod ascii;
mod cea;
mod cea_match;
mod cea_source;

mod collator;
pub use collator::Collator;

#[cfg(feature = "pipeline-stats")]
pub use collator::PipelineStats;

mod consts;
mod first_weight;
mod normalize;
mod prefix;
mod sort_key;
mod tables;

mod types;
pub use types::{Locale, Tailoring};

mod weights;
