use alloc::vec::Vec;

use ibc_client_tendermint::types::proto::v1::ClientState;
use ibc_proto::ibc::lightclients::tendermint::v1::Header as TmHeader;

/// ClientState from Tendermint tracks the current validator set, latest height,
/// and a possible frozen height.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SovTmClientState {
    #[prost(message, optional, tag = "1")]
    pub da_client_state: ::core::option::Option<ClientState>,
    #[prost(message, optional, tag = "2")]
    pub rollup_client_state: ::core::option::Option<RollupClientState>,
}

#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RollupClientState {
    #[prost(string, tag = "1")]
    pub rollup_id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub post_root_state: ::prost::alloc::vec::Vec<u8>,
}

#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[derive(Clone, PartialEq, prost::Message)]
pub struct SovTendermintHeader {
    #[prost(message, optional, tag = "1")]
    pub core_header: ::core::option::Option<TmHeader>,
    #[prost(message, optional, tag = "2")]
    pub aggregated_proof: ::core::option::Option<AggregatedProof>,
}

#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[derive(Clone, PartialEq, prost::Message)]
pub struct AggregatedProof {
    #[prost(message, optional, tag = "1")]
    pub public_input: ::core::option::Option<PublicInput>,
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
}

#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[derive(Clone, PartialEq, prost::Message)]
pub struct PublicInput {
    #[prost(message, optional, tag = "1")]
    pub initial_da_block_hash: ::core::option::Option<Vec<u8>>,
    #[prost(message, optional, tag = "2")]
    pub final_da_block_hash: ::core::option::Option<Vec<u8>>,
    #[prost(message, optional, tag = "3")]
    pub initial_state_root: ::core::option::Option<Vec<u8>>,
    #[prost(message, optional, tag = "4")]
    pub post_state_root: ::core::option::Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct SovTendermintMisbehaviour {
    #[prost(string, tag = "1")]
    pub client_id: String,
    #[prost(message, optional, tag = "2")]
    pub header_1: Option<SovTendermintHeader>,
    #[prost(message, optional, tag = "3")]
    pub header_2: Option<SovTendermintHeader>,
}
