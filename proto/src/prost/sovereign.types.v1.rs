/// AggregatedProof is the overarching structure, encompassing public data, proof
/// data information, and aggregated proof bytes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AggregatedProof {
    /// the public data of the aggregated proof
    #[prost(message, optional, tag = "1")]
    pub public_data: ::core::option::Option<AggregatedProofPublicData>,
    /// the aggregated proof bytes
    #[prost(message, optional, tag = "2")]
    pub serialized_proof: ::core::option::Option<SerializedAggregatedProof>,
}
impl ::prost::Name for AggregatedProof {
    const NAME: &'static str = "AggregatedProof";
    const PACKAGE: &'static str = "sovereign.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("sovereign.types.v1.{}", Self::NAME)
    }
}
/// AggregatedProofPublicData defines the public properties of the
/// AggregatedProof for the Sovereign SDK rollups, utilized for verifying the
/// proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AggregatedProofPublicData {
    /// The set of validity conditions for each block of the aggregated proof.
    #[prost(message, repeated, tag = "1")]
    pub validity_conditions: ::prost::alloc::vec::Vec<SerializedValidityCondition>,
    /// the initial slot number of the rollup from which the proof captures the
    /// rollup's transition from the initial state root.
    #[prost(uint64, tag = "2")]
    pub initial_slot_number: u64,
    /// the final slot number of the rollup, up to which the proof captures the
    /// rollup's transition to the final state root.
    #[prost(uint64, tag = "3")]
    pub final_slot_number: u64,
    /// the genesis state root
    #[prost(bytes = "vec", tag = "4")]
    pub genesis_state_root: ::prost::alloc::vec::Vec<u8>,
    /// the initial state root
    #[prost(bytes = "vec", tag = "5")]
    pub initial_state_root: ::prost::alloc::vec::Vec<u8>,
    /// the final state root
    #[prost(bytes = "vec", tag = "6")]
    pub final_state_root: ::prost::alloc::vec::Vec<u8>,
    /// the initial slot hash
    #[prost(bytes = "vec", tag = "7")]
    pub initial_slot_hash: ::prost::alloc::vec::Vec<u8>,
    /// the final slot hash
    #[prost(bytes = "vec", tag = "8")]
    pub final_slot_hash: ::prost::alloc::vec::Vec<u8>,
    /// the code commitment of the aggregated proof circuit
    #[prost(message, optional, tag = "9")]
    pub code_commitment: ::core::option::Option<CodeCommitment>,
}
impl ::prost::Name for AggregatedProofPublicData {
    const NAME: &'static str = "AggregatedProofPublicData";
    const PACKAGE: &'static str = "sovereign.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("sovereign.types.v1.{}", Self::NAME)
    }
}
/// SerializedAggregatedProof defines the raw aggregated proof structure for the
/// Sovereign SDK rollups.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SerializedAggregatedProof {
    /// the rollup raw aggregated proof bytes covering a range of DA blocks
    #[prost(bytes = "vec", tag = "1")]
    pub raw_aggregated_proof: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SerializedAggregatedProof {
    const NAME: &'static str = "SerializedAggregatedProof";
    const PACKAGE: &'static str = "sovereign.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("sovereign.types.v1.{}", Self::NAME)
    }
}
/// SerializedValidityCondition defines the serialized validity condition for
/// each block of the aggregated proof
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SerializedValidityCondition {
    /// the validity condition
    #[prost(bytes = "vec", tag = "1")]
    pub validity_condition: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SerializedValidityCondition {
    const NAME: &'static str = "SerializedValidityCondition";
    const PACKAGE: &'static str = "sovereign.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("sovereign.types.v1.{}", Self::NAME)
    }
}
/// CodeCommitment defines the code commitment of the aggregated proof circuit
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CodeCommitment {
    /// the code commitment
    #[prost(bytes = "vec", tag = "1")]
    pub code_commitment: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for CodeCommitment {
    const NAME: &'static str = "CodeCommitment";
    const PACKAGE: &'static str = "sovereign.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("sovereign.types.v1.{}", Self::NAME)
    }
}
