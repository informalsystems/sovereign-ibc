#![cfg_attr(not(test), deny(clippy::unwrap_used))]
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

pub mod client_state;
pub mod consensus_state;

/// Re-export of Sovereign light client data structures from `sov-celestia-client-types` crate.
pub mod types {
    #[doc(inline)]
    pub use sov_celestia_client_types::*;
}
