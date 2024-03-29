mod aggregated_proof;
mod header;
mod misbehaviour;
mod pretty;

use core::fmt::Debug;

pub use aggregated_proof::*;
pub use header::*;
use ibc_client_tendermint::types::Header as TmHeader;
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::proto::{Any, Protobuf};
pub use misbehaviour::*;
use prost::Message;

use crate::error::Error;

/// Defines the union ClientMessage type allowing to submit all possible
/// messages for updating clients or reporting misbehaviour.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientMessage<H>
where
    H: Clone + Debug,
{
    Header(Box<Header<H>>),
    Misbehaviour(Box<Misbehaviour<H>>),
}

/// ClientMessage type alias for the Sovereign SDK rollups operating on the
/// Tendermint-driven DA layer.
pub type SovTmClientMessage = ClientMessage<TmHeader>;

impl SovTmClientMessage {
    /// Decodes a `SovTmClientMessage` from a byte array using the `Any` type.
    pub fn decode(value: Vec<u8>) -> Result<SovTmClientMessage, Error> {
        let any = Any::decode(&mut value.as_slice()).map_err(Error::source)?;
        SovTmClientMessage::try_from(any)
    }
}

impl Protobuf<Any> for SovTmClientMessage {}

impl TryFrom<Any> for SovTmClientMessage {
    type Error = Error;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        let msg = match &*any.type_url {
            SOV_TENDERMINT_HEADER_TYPE_URL => Self::Header(Box::new(SovTmHeader::try_from(any)?)),
            SOV_TENDERMINT_MISBEHAVIOUR_TYPE_URL => {
                Self::Misbehaviour(Box::new(SovTmMisbehaviour::try_from(any)?))
            }
            _ => Err(Error::invalid(format!("Unknown type: {}", any.type_url)))?,
        };

        Ok(msg)
    }
}

impl From<SovTmClientMessage> for Any {
    fn from(msg: SovTmClientMessage) -> Self {
        match msg {
            ClientMessage::Header(header) => Any {
                type_url: SOV_TENDERMINT_HEADER_TYPE_URL.to_string(),
                value: Protobuf::<Any>::encode_vec(*header),
            },
            ClientMessage::Misbehaviour(misbehaviour) => Any {
                type_url: SOV_TENDERMINT_MISBEHAVIOUR_TYPE_URL.to_string(),
                value: Protobuf::<Any>::encode_vec(*misbehaviour),
            },
        }
    }
}

#[cfg(feature = "test-util")]
pub mod test_util {
    use ibc_client_tendermint::types::Header as TmHeader;
    use ibc_core::client::types::Height;

    use super::*;
    use crate::client_state::test_util::HeaderConfig;
    use crate::proto::types::v1::AggregatedProof as RawAggregatedProof;

    #[derive(typed_builder::TypedBuilder, Debug)]
    #[builder(build_method(into = AggregatedProofData))]
    pub struct AggregatedProofDataConfig {
        pub public_input: PublicInputConfig,
        #[builder(default)]
        pub aggregated_proof: AggregatedProofConfig,
    }

    impl From<AggregatedProofDataConfig> for AggregatedProofData {
        fn from(config: AggregatedProofDataConfig) -> Self {
            Self {
                public_input: config.public_input.into(),
                aggregated_proof: config.aggregated_proof.into(),
            }
        }
    }

    #[derive(typed_builder::TypedBuilder, Debug)]
    pub struct PublicInputConfig {
        #[builder(default)]
        pub validity_conditions: Vec<ValidityCondition>,
        pub initial_slot_number: Height,
        pub final_slot_number: Height,
        #[builder(default = Root::from([0; 32]))]
        pub genesis_state_root: Root,
        #[builder(default = Root::from([0; 32]))]
        pub input_state_root: Root,
        #[builder(default = Root::from([0; 32]))]
        pub final_state_root: Root,
        #[builder(default)]
        pub initial_slot_hash: Vec<u8>,
        #[builder(default)]
        pub final_slot_hash: Vec<u8>,
        #[builder(default = CodeCommitment::from(vec![0; 32]))]
        pub code_commitment: CodeCommitment,
    }

    impl From<PublicInputConfig> for AggregatedProofPublicInput {
        fn from(config: PublicInputConfig) -> Self {
            Self {
                validity_conditions: config.validity_conditions,
                initial_slot_number: config.initial_slot_number,
                final_slot_number: config.final_slot_number,
                genesis_state_root: config.genesis_state_root,
                input_state_root: config.input_state_root,
                final_state_root: config.final_state_root,
                initial_slot_hash: config.initial_slot_hash,
                final_slot_hash: config.final_slot_hash,
                code_commitment: config.code_commitment,
            }
        }
    }

    #[derive(typed_builder::TypedBuilder, Debug, Default)]
    pub struct AggregatedProofConfig {
        #[builder(default)]
        pub proof: Vec<u8>,
    }

    impl From<AggregatedProofConfig> for AggregatedProof {
        fn from(config: AggregatedProofConfig) -> Self {
            Self::from(RawAggregatedProof {
                proof: config.proof,
            })
        }
    }

    pub fn dummy_sov_header(
        da_header: TmHeader,
        initial_slot_number: Height,
        final_slot_number: Height,
        final_state_root: Root,
    ) -> SovTmHeader {
        let aggregated_proof_data = AggregatedProofDataConfig::builder()
            .public_input(
                PublicInputConfig::builder()
                    .initial_slot_number(initial_slot_number)
                    .final_slot_number(final_slot_number)
                    .final_state_root(final_state_root)
                    .build(),
            )
            .build();

        HeaderConfig::builder()
            .da_header(da_header)
            .aggregated_proof_data(aggregated_proof_data)
            .build()
    }
}
