//! Defines the aggregated proof data structures, and their conversions to and
//! from the raw Protobuf types for the Sovereign SDK rollups.
//!
//! Note: Since Rust protobuf types currently live in `sovereign-ibc`,
//! additionally we are in the midst of development where aggregated proof
//! definitions are evolving, and want to leverage client-specific methods and
//! implementations. As a result, we're keeping a set of domain types identical
//! to those in the Sovereign SDK, at least for now. This facilitates easier
//! development and minimizes dependencies on the Sovereign SDK repository.
//! Looking ahead, we may consider merging these two into a potential shared
//! client-side library.

use core::fmt::{Display, Error as FmtError, Formatter};

use ibc_core::client::types::Height;
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::proto::Protobuf;
use sov_rollup_interface::zk::aggregated_proof::{
    AggregatedProof as SovAggregatedProof,
    AggregatedProofPublicData as SovAggregatedProofPublicData, CodeCommitment as SovCodeCommitment,
    SerializedAggregatedProof as SovSerializedAggregatedProof,
};

use crate::client_message::pretty::PrettySlice;
use crate::error::Error;
use crate::proto::types::v1::{
    AggregatedProof as RawAggregatedProof,
    AggregatedProofPublicData as RawAggregatedProofPublicData, CodeCommitment as RawCodeCommitment,
    SerializedAggregatedProof as RawSerializedAggregatedProof,
    SerializedValidityCondition as RawSerializedValidityCondition,
};

/// Defines the aggregated proof data structure for the Sovereign SDK rollups
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AggregatedProof {
    pub public_data: AggregatedProofPublicData,
    pub serialized_proof: SerializedAggregatedProof,
}

impl AggregatedProof {
    pub fn new(
        public_data: AggregatedProofPublicData,
        serialized_proof: SerializedAggregatedProof,
    ) -> Self {
        Self {
            public_data,
            serialized_proof,
        }
    }

    pub fn public_data(&self) -> &AggregatedProofPublicData {
        &self.public_data
    }

    pub fn serialized_proof(&self) -> &SerializedAggregatedProof {
        &self.serialized_proof
    }

    pub fn validate_basic(&self) -> Result<(), Error> {
        self.public_data.basic_validate()?;

        if self.serialized_proof.is_empty() {
            return Err(Error::empty("serialized proof"));
        }

        Ok(())
    }
}

impl Display for AggregatedProof {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "AggregatedProof {{ aggregated_proof_public_data: {}, serialized_proof: {} }}",
            &self.public_data, self.serialized_proof
        )
    }
}

impl Protobuf<RawAggregatedProof> for AggregatedProof {}

impl TryFrom<RawAggregatedProof> for AggregatedProof {
    type Error = Error;

    fn try_from(raw: RawAggregatedProof) -> Result<Self, Self::Error> {
        Ok(Self {
            public_data: raw
                .public_data
                .ok_or(Error::missing("public data"))?
                .try_into()?,
            serialized_proof: raw
                .serialized_proof
                .ok_or(Error::missing("serialized proof"))?
                .into(),
        })
    }
}

impl From<AggregatedProof> for RawAggregatedProof {
    fn from(value: AggregatedProof) -> Self {
        Self {
            public_data: Some(value.public_data.into()),
            serialized_proof: Some(value.serialized_proof.into()),
        }
    }
}

impl From<AggregatedProof> for SovAggregatedProof {
    fn from(value: AggregatedProof) -> Self {
        Self::new(value.serialized_proof.into(), value.public_data.into())
    }
}

/// Defines the public properties of the AggregatedProof for the Sovereign SDK
/// rollups, utilized for verifying the proof.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AggregatedProofPublicData {
    pub validity_conditions: Vec<ValidityCondition>,
    pub initial_slot_number: Height,
    pub final_slot_number: Height,
    pub genesis_state_root: Root,
    pub initial_state_root: Root,
    pub final_state_root: Root,
    pub initial_slot_hash: Vec<u8>,
    pub final_slot_hash: Vec<u8>,
    pub code_commitment: CodeCommitment,
}

impl AggregatedProofPublicData {
    pub fn initial_slot_number(&self) -> Height {
        self.initial_slot_number
    }

    pub fn final_slot_number(&self) -> Height {
        self.final_slot_number
    }

    pub fn genesis_state_root(&self) -> &Root {
        &self.genesis_state_root
    }

    pub fn final_state_root(&self) -> &Root {
        &self.final_state_root
    }

    pub fn code_commitment(&self) -> &CodeCommitment {
        &self.code_commitment
    }

    pub fn basic_validate(&self) -> Result<(), Error> {
        if self.validity_conditions.is_empty() {
            return Err(Error::empty("validity conditions"));
        }

        self.validity_conditions.iter().try_for_each(|vc| {
            if vc.is_empty() {
                return Err(Error::empty("validity condition"));
            }
            Ok(())
        })?;

        if self.initial_slot_number > self.final_slot_number {
            return Err(Error::invalid(
                "initial slot number is greater than final slot number",
            ));
        }

        if self.initial_slot_hash.is_empty() {
            return Err(Error::empty("initial slot hash"));
        }

        if self.final_slot_hash.is_empty() {
            return Err(Error::empty("final slot hash"));
        }

        if self.code_commitment.is_empty() {
            return Err(Error::empty("code commitment"));
        }

        Ok(())
    }
}

impl Display for AggregatedProofPublicData {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
                f,
                "AggregatedProofPublicData {{ validity_conditions: {}, initial_slot_number: {}, final_slot_number: {}, initial_slot_hash: {}, final_slot_hash: {}, genesis_state_root: {}, initial_state_root: {}, final_state_root: {}, code_commitment: {} }}",
                PrettySlice(&self.validity_conditions),
                self.initial_slot_number,
                self.final_slot_number,
                hex::encode(self.genesis_state_root.as_ref()),
                hex::encode(self.initial_state_root.as_ref()),
                hex::encode(self.final_state_root.as_ref()),
                hex::encode(&self.initial_slot_hash),
                hex::encode(&self.final_slot_hash),
                self.code_commitment,
            )
    }
}

impl From<AggregatedProofPublicData> for SovAggregatedProofPublicData {
    fn from(value: AggregatedProofPublicData) -> Self {
        Self {
            validity_conditions: value
                .validity_conditions
                .into_iter()
                .map(|vc| vc.0)
                .collect(),
            initial_slot_number: value.initial_slot_number.revision_height(),
            final_slot_number: value.final_slot_number.revision_height(),
            genesis_state_root: value.genesis_state_root.into(),
            initial_state_root: value.initial_state_root.into(),
            final_state_root: value.final_state_root.into(),
            initial_slot_hash: value.initial_slot_hash,
            final_slot_hash: value.final_slot_hash,
            code_commitment: value.code_commitment.into(),
        }
    }
}

impl Protobuf<RawAggregatedProofPublicData> for AggregatedProofPublicData {}

impl TryFrom<RawAggregatedProofPublicData> for AggregatedProofPublicData {
    type Error = Error;

    fn try_from(raw: RawAggregatedProofPublicData) -> Result<Self, Self::Error> {
        Ok(Self {
            validity_conditions: raw
                .validity_conditions
                .into_iter()
                .map(Into::into)
                .collect(),
            initial_slot_number: Height::new(0, raw.initial_slot_number)?,
            final_slot_number: Height::new(0, raw.final_slot_number)?,
            genesis_state_root: raw.genesis_state_root.try_into()?,
            initial_state_root: raw.initial_state_root.try_into()?,
            final_state_root: raw.final_state_root.try_into()?,
            initial_slot_hash: raw.initial_slot_hash,
            final_slot_hash: raw.final_slot_hash,
            code_commitment: raw
                .code_commitment
                .ok_or(Error::missing("code commitment"))?
                .into(),
        })
    }
}

impl From<AggregatedProofPublicData> for RawAggregatedProofPublicData {
    fn from(value: AggregatedProofPublicData) -> Self {
        Self {
            validity_conditions: value
                .validity_conditions
                .into_iter()
                .map(Into::into)
                .collect(),
            initial_slot_number: value.initial_slot_number.revision_height(),
            final_slot_number: value.final_slot_number.revision_height(),
            genesis_state_root: value.genesis_state_root.into(),
            initial_state_root: value.initial_state_root.into(),
            final_state_root: value.final_state_root.into(),
            initial_slot_hash: value.initial_slot_hash,
            final_slot_hash: value.final_slot_hash,
            code_commitment: Some(value.code_commitment.into()),
        }
    }
}

/// Defines the validity condition for each block of the aggregated proof
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidityCondition(Vec<u8>);

impl ValidityCondition {
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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

impl Protobuf<RawSerializedValidityCondition> for ValidityCondition {}

impl From<RawSerializedValidityCondition> for ValidityCondition {
    fn from(raw: RawSerializedValidityCondition) -> Self {
        Self(raw.validity_condition)
    }
}

impl From<ValidityCondition> for RawSerializedValidityCondition {
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
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
            code_commitment: value.0.to_vec(),
        }
    }
}

impl From<CodeCommitment> for SovCodeCommitment {
    fn from(value: CodeCommitment) -> Self {
        Self(value.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SerializedAggregatedProof(Vec<u8>);

impl SerializedAggregatedProof {
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for SerializedAggregatedProof {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if self.0.is_empty() {
            return write!(f, "SerializedAggregatedProof([])");
        }
        write!(f, "SerializedAggregatedProof(0x{})", hex::encode(&self.0))
    }
}

impl From<Vec<u8>> for SerializedAggregatedProof {
    fn from(proof: Vec<u8>) -> Self {
        Self(proof)
    }
}

impl Protobuf<RawSerializedAggregatedProof> for SerializedAggregatedProof {}

impl From<RawSerializedAggregatedProof> for SerializedAggregatedProof {
    fn from(raw: RawSerializedAggregatedProof) -> Self {
        Self(raw.raw_aggregated_proof)
    }
}

impl From<SerializedAggregatedProof> for RawSerializedAggregatedProof {
    fn from(value: SerializedAggregatedProof) -> Self {
        Self {
            raw_aggregated_proof: value.0,
        }
    }
}

impl From<SerializedAggregatedProof> for SovSerializedAggregatedProof {
    fn from(value: SerializedAggregatedProof) -> Self {
        Self {
            raw_aggregated_proof: value.0,
        }
    }
}

/// Defines the root hash of the aggregated proof
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Root([u8; 32]);

impl AsRef<[u8; 32]> for Root {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl TryFrom<Vec<u8>> for Root {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let root = value.as_slice().try_into().map_err(Error::source)?;

        Ok(Self(root))
    }
}

impl From<[u8; 32]> for Root {
    fn from(root: [u8; 32]) -> Self {
        Self(root)
    }
}

impl From<jmt::RootHash> for Root {
    fn from(app_hash: jmt::RootHash) -> Self {
        Self::from(app_hash.0)
    }
}

impl From<Root> for Vec<u8> {
    fn from(root: Root) -> Self {
        root.0.to_vec()
    }
}
