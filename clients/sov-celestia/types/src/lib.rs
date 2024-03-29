extern crate alloc;

pub mod client_message;
pub mod client_state;
pub mod codec;
pub mod consensus_state;
pub mod error;

/// Re-exports Sovereign SDK rollup light clients proto types from
/// `ibc_proto-rs`
pub mod proto {
    pub use sov_ibc_proto::ibc::lightclients::sovereign::*;
    pub use sov_ibc_proto::sovereign::*;
}
