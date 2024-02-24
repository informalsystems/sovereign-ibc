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
use ibc_proto::ibc::lightclients::sovereign::tendermint::v1::{
    Header as RawSovTmHeader, Misbehaviour as RawSovTmMisbehaviour,
};
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
            SOV_TENDERMINT_HEADER_TYPE_URL => {
                let header =
                    Protobuf::<RawSovTmHeader>::decode(&*any.value).map_err(Error::source)?;
                Self::Header(Box::new(header))
            }
            SOV_TENDERMINT_MISBEHAVIOUR_TYPE_URL => {
                let misbehaviour =
                    Protobuf::<RawSovTmMisbehaviour>::decode(&*any.value).map_err(Error::source)?;
                Self::Misbehaviour(Box::new(misbehaviour))
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

    use super::{AggregatedProof, AggregatedProofData, ProofDataInfo, PublicInput, SovTmHeader};
    use crate::client_state::test_util::HeaderConfig;
    use crate::proto::types::v1::AggregatedProof as RawAggregatedProof;

    #[derive(typed_builder::TypedBuilder, Debug)]
    #[builder(build_method(into = AggregatedProofData))]
    pub struct AggregatedProofDataConfig {
        #[builder(default)]
        pub public_input: PublicInputConfig,
        pub proof_data_info: ProofDataInfoConfig,
        #[builder(default)]
        pub aggregated_proof: AggregatedProofConfig,
    }

    impl From<AggregatedProofDataConfig> for AggregatedProofData {
        fn from(config: AggregatedProofDataConfig) -> Self {
            Self {
                public_input: config.public_input.into(),
                proof_data_info: config.proof_data_info.into(),
                aggregated_proof: config.aggregated_proof.into(),
            }
        }
    }

    #[derive(typed_builder::TypedBuilder, Debug, Default)]
    pub struct PublicInputConfig {
        #[builder(default)]
        pub initial_da_block_hash: Vec<u8>,
        #[builder(default)]
        pub final_da_block_hash: Vec<u8>,
        #[builder(default)]
        pub genesis_state_root: Vec<u8>,
        #[builder(default)]
        pub input_state_root: Vec<u8>,
        #[builder(default)]
        pub final_state_root: Vec<u8>,
        #[builder(default)]
        pub code_commitment: Vec<u8>,
    }

    impl From<PublicInputConfig> for PublicInput {
        fn from(config: PublicInputConfig) -> Self {
            Self {
                initial_da_block_hash: config.initial_da_block_hash,
                final_da_block_hash: config.final_da_block_hash,
                genesis_state_root: config.genesis_state_root,
                input_state_root: config.input_state_root,
                final_state_root: config.final_state_root,
                code_commitment: config.code_commitment,
            }
        }
    }

    #[derive(typed_builder::TypedBuilder, Debug)]
    pub struct ProofDataInfoConfig {
        pub initial_state_height: Height,
        pub final_state_height: Height,
    }

    impl From<ProofDataInfoConfig> for ProofDataInfo {
        fn from(config: ProofDataInfoConfig) -> Self {
            Self {
                initial_state_height: config.initial_state_height,
                final_state_height: config.final_state_height,
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
        initial_state_height: Height,
        final_state_height: Height,
    ) -> SovTmHeader {
        let aggregated_proof_data = AggregatedProofDataConfig::builder()
            .proof_data_info(
                ProofDataInfoConfig::builder()
                    .initial_state_height(initial_state_height)
                    .final_state_height(final_state_height)
                    .build(),
            )
            .build();

        HeaderConfig::builder()
            .da_header(da_header)
            .aggregated_proof_data(aggregated_proof_data)
            .build()
    }
}
