// This file is @generated by prost-build.
///   SovereignClientParams structure encompasses the essential parameters shared
///   among all Sovereign light clients, regardless of the underlying Data
///   Availability (DA) layer, to track the client state of the rollup.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SovereignClientParams {
    /// the genesis state root of the rollup
    #[prost(bytes = "vec", tag = "1")]
    pub genesis_state_root: ::prost::alloc::vec::Vec<u8>,
    /// the genesis DA height from which the rollup started
    #[prost(message, optional, tag = "2")]
    pub genesis_da_height: ::core::option::Option<
        ::ibc_proto::ibc::core::client::v1::Height,
    >,
    /// the code commitment of the aggregated proof circuit
    #[prost(message, optional, tag = "3")]
    pub code_commitment: ::core::option::Option<
        super::super::super::super::sovereign::types::v1::CodeCommitment,
    >,
    /// the duration of the period since the latest DA Timestamp during which the
    /// submitted headers are valid for upgrade
    #[prost(message, optional, tag = "4")]
    pub trusting_period: ::core::option::Option<::ibc_proto::google::protobuf::Duration>,
    /// the frozen height when the client was frozen due to the misbehaviour
    #[prost(message, optional, tag = "6")]
    pub frozen_height: ::core::option::Option<
        ::ibc_proto::ibc::core::client::v1::Height,
    >,
    /// the latest height (slot number) the client was updated to
    #[prost(message, optional, tag = "5")]
    pub latest_height: ::core::option::Option<
        ::ibc_proto::ibc::core::client::v1::Height,
    >,
    /// the path at which next upgraded client will be committed. Each element
    /// corresponds to the key for a single CommitmentProof in the chained proof.
    /// NOTE: ClientState must stored under
    /// `{upgradePath}/{upgradeHeight}/clientState` ConsensusState must be stored
    /// under `{upgradepath}/{upgradeHeight}/consensusState`
    #[prost(string, tag = "7")]
    pub upgrade_path: ::prost::alloc::string::String,
}
impl ::prost::Name for SovereignClientParams {
    const NAME: &'static str = "SovereignClientParams";
    const PACKAGE: &'static str = "ibc.lightclients.sovereign.v1";
    fn full_name() -> ::prost::alloc::string::String {
        "ibc.lightclients.sovereign.v1.SovereignClientParams".into()
    }
    fn type_url() -> ::prost::alloc::string::String {
        "/ibc.lightclients.sovereign.v1.SovereignClientParams".into()
    }
}
/// SovereignConsensusParams structure encompasses the essential parameters
/// shared among all Sovereign light clients, regardless of the underlying Data
/// Availability (DA) layer, to track the consensus state of the rollup.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SovereignConsensusParams {
    /// the state root of rollup at the ConsensusState height
    #[prost(message, optional, tag = "1")]
    pub root: ::core::option::Option<::ibc_proto::ibc::core::commitment::v1::MerkleRoot>,
}
impl ::prost::Name for SovereignConsensusParams {
    const NAME: &'static str = "SovereignConsensusParams";
    const PACKAGE: &'static str = "ibc.lightclients.sovereign.v1";
    fn full_name() -> ::prost::alloc::string::String {
        "ibc.lightclients.sovereign.v1.SovereignConsensusParams".into()
    }
    fn type_url() -> ::prost::alloc::string::String {
        "/ibc.lightclients.sovereign.v1.SovereignConsensusParams".into()
    }
}