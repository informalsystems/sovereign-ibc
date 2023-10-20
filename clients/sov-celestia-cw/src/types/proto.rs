use alloc::vec::Vec;

use celestia_proto::header::pb::ExtendedHeader;
use ibc::proto::tendermint::v1::Fraction;
use ibc_proto::google::protobuf::{Duration, Timestamp};
use ibc_proto::ibc::core::client::v1::Height;
use ibc_proto::ibc::core::commitment::v1::MerkleRoot;
use ics23::ProofSpec;
use tendermint_proto::v0_34::types::ValidatorSet;

#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[derive(Clone, PartialEq, prost::Message)]
pub struct SovHeader {
    #[prost(message, optional, tag = "1")]
    pub da_header: ::core::option::Option<CelestiaHeader>,
    #[prost(message, optional, tag = "2")]
    pub aggregate_snark: ::core::option::Option<AggregateSNARK>,
}

#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[derive(Clone, PartialEq, prost::Message)]
pub struct AggregateSNARK {
    #[prost(message, optional, tag = "1")]
    pub start_da_hash: ::core::option::Option<Vec<u8>>,
    #[prost(message, optional, tag = "2")]
    pub end_da_hash: ::core::option::Option<Vec<u8>>,
    #[prost(message, optional, tag = "3")]
    pub input_state_root: ::core::option::Option<Vec<u8>>,
    #[prost(message, optional, tag = "4")]
    pub output_state_root: ::core::option::Option<Vec<u8>>,
}

#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[derive(Clone, PartialEq, prost::Message)]
pub struct CelestiaHeader {
    #[prost(message, optional, tag = "1")]
    pub extended_header: ::core::option::Option<ExtendedHeader>,
    #[prost(message, optional, tag = "2")]
    pub trusted_height: ::core::option::Option<Height>,
    #[prost(message, optional, tag = "3")]
    pub trusted_next_validator_set: Option<ValidatorSet>,
}

/// ClientState from Tendermint tracks the current validator set, latest height,
/// and a possible frozen height.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub trust_level: ::core::option::Option<Fraction>,
    /// duration of the period since the LastestTimestamp during which the
    /// submitted headers are valid for upgrade
    #[prost(message, optional, tag = "3")]
    pub trusting_period: ::core::option::Option<Duration>,
    /// duration of the staking unbonding period
    #[prost(message, optional, tag = "4")]
    pub unbonding_period: ::core::option::Option<Duration>,
    /// defines how much new (untrusted) header's Time can drift into the future.
    #[prost(message, optional, tag = "5")]
    pub max_clock_drift: ::core::option::Option<Duration>,
    /// Block height when the client was frozen due to a misbehaviour
    #[prost(message, optional, tag = "6")]
    pub frozen_height: ::core::option::Option<Height>,
    /// Latest height the client was updated to
    #[prost(message, optional, tag = "7")]
    pub latest_height: ::core::option::Option<Height>,
    /// Proof specifications used in verifying counterparty state
    #[prost(message, repeated, tag = "8")]
    pub proof_specs: ::prost::alloc::vec::Vec<ProofSpec>,
    /// Path at which next upgraded client will be committed.
    /// Each element corresponds to the key for a single CommitmentProof in the
    /// chained proof. NOTE: ClientState must stored under
    /// `{upgradePath}/{upgradeHeight}/clientState` ConsensusState must be stored
    /// under `{upgradepath}/{upgradeHeight}/consensusState` For SDK chains using
    /// the default upgrade module, upgrade_path should be \[\]string{"upgrade",
    /// "upgradedIBCState"}`
    #[prost(string, repeated, tag = "9")]
    pub upgrade_path: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// allow_update_after_expiry is deprecated
    #[deprecated]
    #[prost(bool, tag = "10")]
    pub allow_update_after_expiry: bool,
    /// allow_update_after_misbehaviour is deprecated
    #[deprecated]
    #[prost(bool, tag = "11")]
    pub allow_update_after_misbehaviour: bool,
}
/// ConsensusState defines the consensus state from Tendermint.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    /// timestamp that corresponds to the block height in which the ConsensusState
    /// was stored.
    #[prost(message, optional, tag = "1")]
    pub timestamp: ::core::option::Option<Timestamp>,
    /// commitment root (i.e app hash)
    #[prost(message, optional, tag = "2")]
    pub root: ::core::option::Option<MerkleRoot>,
    #[prost(bytes = "vec", tag = "3")]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
}
