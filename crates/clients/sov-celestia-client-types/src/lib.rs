extern crate alloc;

pub mod client_message;
pub mod client_state;
pub mod consensus_state;

/// Re-exports Sovereign SDK light clients types from the `sov_ibc_client_types`
/// crate.
pub mod sovereign {
    pub use sov_client_types::*;
}

/// Re-exports Sovereign SDK rollup light clients proto types from
/// `ibc_proto-rs`
pub mod proto {
    pub use sov_ibc_proto::ibc::lightclients::sovereign::tendermint::*;
}
