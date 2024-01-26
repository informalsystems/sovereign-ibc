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

use crate::error::Error;

/// Defines the union ClientMessage type allowing to submit all possible
/// messages for updating clients or reporting misbehaviour.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientMessage<H>
where
    H: Clone + Debug,
{
    Header(Box<Header<H>>),
    Misbehaviour(Box<SovMisbehaviour<H>>),
}

/// ClientMessage type alias for the Sovereign SDK rollups operating on the
/// Tendermint-driven DA layer.
pub type SovTmClientMessage = ClientMessage<TmHeader>;

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
