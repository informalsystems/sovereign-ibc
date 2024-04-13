mod header;
mod misbehaviour;

use core::fmt::Debug;

pub use header::*;
use ibc_client_tendermint::types::Header as TmHeader;
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::proto::{Any, Protobuf};
pub use misbehaviour::*;
use prost::Message;

use crate::sovereign::Error;

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

    use super::*;
    use crate::client_state::test_util::HeaderConfig;
    use crate::sovereign::{AggregatedProofConfig, PublicDataConfig, Root};

    pub fn dummy_sov_header(
        da_header: TmHeader,
        initial_slot_number: u64,
        final_slot_number: u64,
        final_state_root: Root,
    ) -> SovTmHeader {
        let aggregated_proof = AggregatedProofConfig::builder()
            .public_data(
                PublicDataConfig::builder()
                    .initial_slot_number(initial_slot_number.into())
                    .final_slot_number(final_slot_number.into())
                    .final_state_root(final_state_root)
                    .build(),
            )
            .build();

        HeaderConfig::builder()
            .da_header(da_header)
            .aggregated_proof(aggregated_proof)
            .build()
    }
}
