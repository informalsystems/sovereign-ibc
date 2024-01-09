use alloc::vec::Vec;

use ibc_proto::ibc::lightclients::sovereign::tendermint::v1::{
    AggregatedProof as RawAggregatedProof, PublicInput as RawPublicInput,
};

use crate::error::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AggregatedProof {
    pub public_input: PublicInput,
    pub proof: Vec<u8>,
}

impl TryFrom<RawAggregatedProof> for AggregatedProof {
    type Error = Error;

    fn try_from(raw: RawAggregatedProof) -> Result<Self, Self::Error> {
        Ok(Self {
            public_input: raw
                .public_input
                .ok_or(Error::missing("public_input"))?
                .try_into()?,
            proof: raw.proof,
        })
    }
}

impl From<AggregatedProof> for RawAggregatedProof {
    fn from(value: AggregatedProof) -> Self {
        Self {
            public_input: Some(value.public_input.into()),
            proof: value.proof,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PublicInput {
    pub initial_da_block_hash: Vec<u8>,
    pub final_da_block_hash: Vec<u8>,
    pub input_state_root: Vec<u8>,
    pub post_state_root: Vec<u8>,
}

impl TryFrom<RawPublicInput> for PublicInput {
    type Error = Error;

    fn try_from(raw: RawPublicInput) -> Result<Self, Self::Error> {
        Ok(Self {
            initial_da_block_hash: raw.initial_da_block_hash,
            final_da_block_hash: raw.final_da_block_hash,
            input_state_root: raw.inital_state_root,
            post_state_root: raw.post_state_root,
        })
    }
}

impl From<PublicInput> for RawPublicInput {
    fn from(value: PublicInput) -> Self {
        Self {
            initial_da_block_hash: value.initial_da_block_hash,
            final_da_block_hash: value.final_da_block_hash,
            inital_state_root: value.input_state_root,
            post_state_root: value.post_state_root,
        }
    }
}
