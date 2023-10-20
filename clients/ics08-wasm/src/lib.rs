#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;

pub mod client_message;
pub mod client_state;
pub mod consensus_state;
pub mod msg;
pub mod serializer;

pub type Bytes = Vec<u8>;

pub static SUBJECT_PREFIX: &[u8] = "subject/".as_bytes();
pub static SUBSTITUTE_PREFIX: &[u8] = "substitute/".as_bytes();
