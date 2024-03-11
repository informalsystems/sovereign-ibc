use core::fmt::{Display, Error as FmtError, Formatter};

use ibc_core::client::types::Height;
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::proto::Protobuf;
use ibc_proto::sovereign::types::v1::{
    AggregatedProof as RawAggregatedProof, AggregatedProofData as RawAggregatedProofData,
    AggregatedProofPublicInput as RawAggregatedProofPublicInput,
    CodeCommitment as RawCodeCommitment, ValidityCondition as RawValidityCondition,
};

use crate::client_message::pretty::PrettySlice;
use crate::error::Error;

/// Defines the aggregated proof data structure for the Sovereign SDK rollups
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AggregatedProofData {
    pub public_input: AggregatedProofPublicInput,
    pub aggregated_proof: AggregatedProof,
}

impl AggregatedProofData {
    pub fn new(
        public_input: AggregatedProofPublicInput,
        aggregated_proof: AggregatedProof,
    ) -> Self {
        Self {
            public_input,
            aggregated_proof,
        }
    }

    pub fn public_input(&self) -> &AggregatedProofPublicInput {
        &self.public_input
    }

    pub fn aggregated_proof(&self) -> &AggregatedProof {
        &self.aggregated_proof
    }
}

impl Display for AggregatedProofData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "AggregatedProofData {{ aggregated_proof_public_input: {}, aggregated_proof: {} }}",
            &self.public_input, self.aggregated_proof
        )
    }
}

impl Protobuf<RawAggregatedProofData> for AggregatedProofData {}

impl TryFrom<RawAggregatedProofData> for AggregatedProofData {
    type Error = Error;

    fn try_from(raw: RawAggregatedProofData) -> Result<Self, Self::Error> {
        Ok(Self {
            public_input: raw
                .public_input
                .ok_or(Error::missing("public input"))?
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
            aggregated_proof: Some(value.aggregated_proof.into()),
        }
    }
}

/// Defines the public properties of the AggregatedProof for the Sovereign SDK
/// rollups, utilized for verifying the proof.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AggregatedProofPublicInput {
    pub validity_conditions: Vec<ValidityCondition>,
    pub initial_slot_number: Height,
    pub final_slot_number: Height,
    pub initial_da_block_hash: Vec<u8>,
    pub final_da_block_hash: Vec<u8>,
    pub genesis_state_root: Vec<u8>,
    pub input_state_root: Vec<u8>,
    pub final_state_root: Vec<u8>,
    pub code_commitment: CodeCommitment,
}

impl AggregatedProofPublicInput {
    pub fn initial_slot_number(&self) -> Height {
        self.initial_slot_number
    }

    pub fn final_slot_number(&self) -> Height {
        self.final_slot_number
    }

    pub fn genesis_state_root(&self) -> &[u8] {
        &self.genesis_state_root
    }

    pub fn final_state_root(&self) -> &[u8] {
        &self.final_state_root
    }

    pub fn code_commitment(&self) -> &CodeCommitment {
        &self.code_commitment
    }
}

impl Display for AggregatedProofPublicInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
                f,
                "AggregatedProofPublicInput {{ validity_conditions: {}, initial_slot_number: {}, final_slot_number: {}, initial_da_block_hash: {}, final_da_block_hash: {}, genesis_state_root: {}, input_state_root: {}, post_state_root: {}, code_commitment: {} }}",
                PrettySlice(&self.validity_conditions),
                self.initial_slot_number,
                self.final_slot_number,
                hex::encode(&self.initial_da_block_hash),
                hex::encode(&self.final_da_block_hash),
                hex::encode(&self.genesis_state_root),
                hex::encode(&self.input_state_root),
                hex::encode(&self.final_state_root),
                self.code_commitment,
            )
    }
}

impl Protobuf<RawAggregatedProofPublicInput> for AggregatedProofPublicInput {}

impl TryFrom<RawAggregatedProofPublicInput> for AggregatedProofPublicInput {
    type Error = Error;

    fn try_from(raw: RawAggregatedProofPublicInput) -> Result<Self, Self::Error> {
        Ok(Self {
            validity_conditions: raw
                .validity_conditions
                .into_iter()
                .map(Into::into)
                .collect(),
            initial_slot_number: Height::new(0, raw.initial_slot_number)?,
            final_slot_number: Height::new(0, raw.final_slot_number)?,
            initial_da_block_hash: raw.initial_da_block_hash,
            final_da_block_hash: raw.final_da_block_hash,
            genesis_state_root: raw.genesis_state_root,
            input_state_root: raw.initial_state_root,
            final_state_root: raw.final_state_root,
            code_commitment: raw
                .code_commitment
                .ok_or(Error::missing("code commitment"))?
                .into(),
        })
    }
}

impl From<AggregatedProofPublicInput> for RawAggregatedProofPublicInput {
    fn from(value: AggregatedProofPublicInput) -> Self {
        Self {
            validity_conditions: value
                .validity_conditions
                .into_iter()
                .map(Into::into)
                .collect(),
            initial_slot_number: value.initial_slot_number.revision_height(),
            final_slot_number: value.final_slot_number.revision_height(),
            initial_da_block_hash: value.initial_da_block_hash,
            final_da_block_hash: value.final_da_block_hash,
            genesis_state_root: value.genesis_state_root,
            initial_state_root: value.input_state_root,
            final_state_root: value.final_state_root,
            code_commitment: Some(value.code_commitment.into()),
        }
    }
}

/// Defines the validity condition for each block of the aggregated proof
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidityCondition(Vec<u8>);

impl ValidityCondition {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl Display for ValidityCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if self.0.is_empty() {
            return write!(f, "ValidityCondition([])");
        }
        write!(f, "ValidityCondition(0x{})", hex::encode(&self.0))
    }
}

impl From<Vec<u8>> for ValidityCondition {
    fn from(validity_condition: Vec<u8>) -> Self {
        Self(validity_condition)
    }
}

impl Protobuf<RawValidityCondition> for ValidityCondition {}

impl From<RawValidityCondition> for ValidityCondition {
    fn from(raw: RawValidityCondition) -> Self {
        Self(raw.validity_condition)
    }
}

impl From<ValidityCondition> for RawValidityCondition {
    fn from(value: ValidityCondition) -> Self {
        Self {
            validity_condition: value.0,
        }
    }
}

/// Defines the code commitment of the aggregated proof circuit
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CodeCommitment(Vec<u8>);

impl CodeCommitment {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl Display for CodeCommitment {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if self.0.is_empty() {
            return write!(f, "CodeCommitment([])");
        }
        write!(f, "CodeCommitment(0x{})", hex::encode(&self.0))
    }
}

impl From<Vec<u8>> for CodeCommitment {
    fn from(code_commitment: Vec<u8>) -> Self {
        Self(code_commitment)
    }
}

impl Protobuf<RawCodeCommitment> for CodeCommitment {}

impl From<RawCodeCommitment> for CodeCommitment {
    fn from(raw: RawCodeCommitment) -> Self {
        Self(raw.code_commitment)
    }
}

impl From<CodeCommitment> for RawCodeCommitment {
    fn from(value: CodeCommitment) -> Self {
        Self {
            code_commitment: value.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AggregatedProof(Vec<u8>);

impl AggregatedProof {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl Display for AggregatedProof {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if self.0.is_empty() {
            return write!(f, "AggregatedProof([])");
        }
        write!(f, "AggregatedProof(0x{})", hex::encode(&self.0))
    }
}

impl From<Vec<u8>> for AggregatedProof {
    fn from(proof: Vec<u8>) -> Self {
        Self(proof)
    }
}

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
