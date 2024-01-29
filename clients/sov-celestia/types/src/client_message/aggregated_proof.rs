use alloc::vec::Vec;

use ibc_core::client::types::Height;
use ibc_core::primitives::proto::Protobuf;
use ibc_proto::sovereign::types::v1::{
    AggregatedProof as RawAggregatedProof, AggregatedProofData as RawAggregatedProofData,
    ProofDataInfo as RawProofDataInfo, PublicInput as RawPublicInput,
};

use crate::error::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AggregatedProofData {
    pub public_input: PublicInput,
    pub proof_data_info: ProofDataInfo,
    pub aggregated_proof: AggregatedProof,
}

impl TryFrom<RawAggregatedProofData> for AggregatedProofData {
    type Error = Error;

    fn try_from(raw: RawAggregatedProofData) -> Result<Self, Self::Error> {
        Ok(Self {
            public_input: raw
                .public_input
                .ok_or(Error::missing("public input"))?
                .try_into()?,
            proof_data_info: raw
                .proof_data_info
                .ok_or(Error::missing("proof data info"))?
                .try_into()?,
            aggregated_proof: raw
                .aggregated_proof
                .ok_or(Error::missing("aggregated proof"))?
                .into(),
        })
    }
}

impl From<AggregatedProofData> for RawAggregatedProofData {
    fn from(value: AggregatedProofData) -> Self {
        Self {
            public_input: Some(value.public_input.into()),
            proof_data_info: Some(value.proof_data_info.into()),
            aggregated_proof: Some(value.aggregated_proof.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PublicInput {
    pub initial_da_block_hash: Vec<u8>,
    pub final_da_block_hash: Vec<u8>,
    pub input_state_root: Vec<u8>,
    pub final_state_root: Vec<u8>,
}

impl TryFrom<RawPublicInput> for PublicInput {
    type Error = Error;

    fn try_from(raw: RawPublicInput) -> Result<Self, Self::Error> {
        Ok(Self {
            initial_da_block_hash: raw.initial_da_block_hash,
            final_da_block_hash: raw.final_da_block_hash,
            input_state_root: raw.initial_state_root,
            final_state_root: raw.final_state_root,
        })
    }
}

impl From<PublicInput> for RawPublicInput {
    fn from(value: PublicInput) -> Self {
        Self {
            initial_da_block_hash: value.initial_da_block_hash,
            final_da_block_hash: value.final_da_block_hash,
            initial_state_root: value.input_state_root,
            final_state_root: value.final_state_root,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProofDataInfo {
    pub initial_state_height: Height,
    pub final_state_height: Height,
}

impl Protobuf<RawProofDataInfo> for ProofDataInfo {}

impl TryFrom<RawProofDataInfo> for ProofDataInfo {
    type Error = Error;

    fn try_from(raw: RawProofDataInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            initial_state_height: Height::new(0, raw.initial_state_height)?,
            final_state_height: Height::new(0, raw.final_state_height)?,
        })
    }
}

impl From<ProofDataInfo> for RawProofDataInfo {
    fn from(value: ProofDataInfo) -> Self {
        Self {
            initial_state_height: value.initial_state_height.revision_height(),
            final_state_height: value.final_state_height.revision_height(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AggregatedProof(Vec<u8>);

impl Protobuf<RawAggregatedProof> for AggregatedProof {}

impl From<RawAggregatedProof> for AggregatedProof {
    fn from(raw: RawAggregatedProof) -> Self {
        Self(raw.proof)
    }
}

impl From<AggregatedProof> for RawAggregatedProof {
    fn from(value: AggregatedProof) -> Self {
        Self { proof: value.0 }
    }
}
