mod aggregated_proof;
mod client_params;
mod consensus_params;
mod error;

#[cfg(feature = "test-util")]
pub use aggregated_proof::test_util::*;
pub use aggregated_proof::{
    AggregatedProof, AggregatedProofPublicData, CodeCommitment, Root, SerializedAggregatedProof,
    SlotNumber, ValidityCondition,
};
#[cfg(feature = "test-util")]
pub use client_params::test_util::*;
pub use client_params::{SovereignClientParams, UpgradePath};
pub use consensus_params::SovereignConsensusParams;
pub use error::*;

/// Re-exports Sovereign SDK light clients proto types from `sov-ibc-proto`
/// crate.
pub mod proto {
    pub use sov_ibc_proto::ibc::lightclients::sovereign::v1::*;
    pub use sov_ibc_proto::sovereign::*;
}
