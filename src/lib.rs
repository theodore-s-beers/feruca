//! This crate provides a basic implementation of the Unicode Collation Algorithm. There is really
//! just one method, `collate`, belonging to a struct, `Collator`, which sets a few options. Despite
//! the bare-bones API, this implementation conforms to the standard and allows for the use of the
//! CLDR root collation order; so it may indeed be useful, even in this early stage of development.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

mod ascii;
mod cea;
mod cea_utils;

mod collator;
pub use collator::Collator;

mod consts;
mod first_weight;
mod normalize;
mod prefix;
mod sort_key;
mod tailor;

mod types;
pub use types::{Locale, Tailoring};

mod weights;
