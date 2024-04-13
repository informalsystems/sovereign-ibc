//! Defines the aggregated proof data structures, and their conversions to and
//! from the raw Protobuf types for the Sovereign SDK rollups.
//!
//! Note: Since Rust protobuf types currently live in `sovereign-ibc`, we are in
//! the midst of development where aggregated proof definitions are evolving,
//! and additionally we want to have control over client-specific methods and
//! implementations, we're currently keeping a set of domain types identical to
//! those in the Sovereign SDK. This facilitates easier development and
//! minimizes dependencies on the Sovereign SDK repository. Looking ahead, we
//! may consider merging these two into a potential shared client-side library.

use core::fmt::{Display, Error as FmtError, Formatter};

use ibc_core::primitives::prelude::*;
use ibc_core::primitives::proto::Protobuf;
use ibc_core::primitives::utils::PrettySlice;

use crate::error::Error;
use crate::proto::types::v1::{
    AggregatedProof as RawAggregatedProof,
    AggregatedProofPublicData as RawAggregatedProofPublicData, CodeCommitment as RawCodeCommitment,
    SerializedAggregatedProof as RawSerializedAggregatedProof,
    SerializedValidityCondition as RawSerializedValidityCondition, SlotNumber as RawSlotNumber,
};

/// Defines the aggregated proof data structure for the Sovereign SDK rollups.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AggregatedProof {
    /// The public data of the aggregated proof
    pub public_data: AggregatedProofPublicData,
    /// The serialized proof
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

    pub fn initial_slot_number(&self) -> u64 {
        self.public_data.initial_slot_number.0
    }

    pub fn final_slot_number(&self) -> u64 {
        self.public_data.final_slot_number.0
    }

    pub fn genesis_state_root(&self) -> &Root {
        &self.public_data.genesis_state_root
    }

    pub fn final_state_root(&self) -> &Root {
        &self.public_data.final_state_root
    }

    pub fn code_commitment(&self) -> &CodeCommitment {
        &self.public_data.code_commitment
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
                .ok_or(Error::missing("public_data"))?
                .try_into()?,
            serialized_proof: raw
                .serialized_proof
                .ok_or(Error::missing("serialized_proof"))?
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

/// Defines the public properties of the AggregatedProof for the Sovereign SDK
/// rollups, utilized for the proof verification.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AggregatedProofPublicData {
    pub validity_conditions: Vec<ValidityCondition>,
    pub initial_slot_number: SlotNumber,
    pub final_slot_number: SlotNumber,
    pub genesis_state_root: Root,
    pub initial_state_root: Root,
    pub final_state_root: Root,
    pub initial_slot_hash: Vec<u8>,
    pub final_slot_hash: Vec<u8>,
    pub code_commitment: CodeCommitment,
}

impl AggregatedProofPublicData {
    pub fn basic_validate(&self) -> Result<(), Error> {
        if self.validity_conditions.is_empty() {
            return Err(Error::empty("validity_conditions"));
        }

        self.validity_conditions.iter().try_for_each(|vc| {
            if vc.is_empty() {
                return Err(Error::empty("validity_condition"));
            }
            Ok(())
        })?;

        if self.initial_slot_number.is_zero() || self.final_slot_number.is_zero() {
            return Err(Error::invalid("slot number cannot be zero"));
        }

        if self.initial_slot_number > self.final_slot_number {
            return Err(Error::invalid(
                "initial slot number is greater than final slot number",
            ));
        }

        if self.initial_slot_hash.is_empty() {
            return Err(Error::empty("initial_slot_hash"));
        }

        if self.final_slot_hash.is_empty() {
            return Err(Error::empty("final_slot_hash"));
        }

        if self.code_commitment.is_empty() {
            return Err(Error::empty("code_commitment"));
        }

        Ok(())
    }
}

impl Display for AggregatedProofPublicData {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
                f,
                "AggregatedProofPublicData {{ validity_conditions: {}, initial_slot_number: {},\
                final_slot_number: {}, initial_slot_hash: {}, final_slot_hash: {}, genesis_state_root: {},\
                initial_state_root: {}, final_state_root: {}, code_commitment: {} }}",
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
            initial_slot_number: raw
                .initial_slot_number
                .ok_or(Error::missing("initial slot number"))?
                .into(),
            final_slot_number: raw
                .final_slot_number
                .ok_or(Error::missing("final slot number"))?
                .into(),
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
            initial_slot_number: Some(value.initial_slot_number.into()),
            final_slot_number: Some(value.final_slot_number.into()),
            genesis_state_root: value.genesis_state_root.into(),
            initial_state_root: value.initial_state_root.into(),
            final_state_root: value.final_state_root.into(),
            initial_slot_hash: value.initial_slot_hash,
            final_slot_hash: value.final_slot_hash,
            code_commitment: Some(value.code_commitment.into()),
        }
    }
}

/// Defines the validity condition for each block of the aggregated proof.
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

/// Defines the code commitment of the aggregated proof circuit.
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

    pub fn matches(&self, other: &CodeCommitment) -> bool {
        self.0 == other.0
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

/// Defines the serialized aggregated proof for the Sovereign SDK rollups.
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

/// Defines the slot number for rollups which is equivalent to the height in the
/// Sovereign SDK system.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, derive_more::Display, derive_more::From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SlotNumber(pub u64);

impl SlotNumber {
    pub fn new(slot_number: u64) -> Self {
        Self(slot_number)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl Protobuf<RawSlotNumber> for SlotNumber {}

impl From<RawSlotNumber> for SlotNumber {
    fn from(raw: RawSlotNumber) -> Self {
        Self(raw.slot_number)
    }
}

impl From<SlotNumber> for RawSlotNumber {
    fn from(value: SlotNumber) -> Self {
        Self {
            slot_number: value.0,
        }
    }
}

/// Defines the root hash of the aggregated proof.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Root([u8; 32]);

impl Root {
    pub fn matches(&self, other: &Root) -> bool {
        self.0 == other.0
    }
}

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

#[cfg(feature = "test-util")]
pub mod test_util {
    use super::*;

    // -------------------------------------------------------------------------
    // NOTE: Vectors default to 32-byte arrays as empty vectors aren't valid.
    // -------------------------------------------------------------------------

    #[derive(typed_builder::TypedBuilder, Debug)]
    #[builder(build_method(into = AggregatedProof))]
    pub struct AggregatedProofConfig {
        pub public_data: PublicDataConfig,
        #[builder(default = vec![0; 32].into())]
        pub serialized_proof: SerializedAggregatedProof,
    }

    impl From<AggregatedProofConfig> for AggregatedProof {
        fn from(config: AggregatedProofConfig) -> Self {
            Self {
                public_data: config.public_data.into(),
                serialized_proof: config.serialized_proof,
            }
        }
    }

    #[derive(typed_builder::TypedBuilder, Debug)]
    pub struct PublicDataConfig {
        #[builder(default = vec![vec![0; 32].into()])]
        pub validity_conditions: Vec<ValidityCondition>,
        pub initial_slot_number: SlotNumber,
        pub final_slot_number: SlotNumber,
        #[builder(default = Root::from([0; 32]))]
        pub genesis_state_root: Root,
        #[builder(default = Root::from([0; 32]))]
        pub initial_state_root: Root,
        #[builder(default = Root::from([0; 32]))]
        pub final_state_root: Root,
        #[builder(default = vec![0; 32])]
        pub initial_slot_hash: Vec<u8>,
        #[builder(default = vec![0; 32])]
        pub final_slot_hash: Vec<u8>,
        #[builder(default = CodeCommitment::from(vec![1; 32]))]
        pub code_commitment: CodeCommitment,
    }

    impl From<PublicDataConfig> for AggregatedProofPublicData {
        fn from(config: PublicDataConfig) -> Self {
            Self {
                validity_conditions: config.validity_conditions,
                initial_slot_number: config.initial_slot_number,
                final_slot_number: config.final_slot_number,
                genesis_state_root: config.genesis_state_root,
                initial_state_root: config.initial_state_root,
                final_state_root: config.final_state_root,
                initial_slot_hash: config.initial_slot_hash,
                final_slot_hash: config.final_slot_hash,
                code_commitment: config.code_commitment,
            }
        }
    }
}
