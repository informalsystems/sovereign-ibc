mod aggregate_snark;
mod celestia_header;
pub mod pretty;
mod sov_header;
mod sov_misbehaviour;

use alloc::format;

pub use aggregate_snark::AggregateSNARK;
pub use celestia_header::CelestiaHeader;
use ibc::proto::Any;
pub use sov_header::{SovHeader, SOVEREIGN_HEADER_TYPE_URL};
pub use sov_misbehaviour::{RawSovMisbehaviour, SovMisbehaviour, SOVEREIGN_MISBEHAVIOUR_TYPE_URL};
use tendermint_proto::Protobuf;

use crate::contract::error::ContractError;
use crate::types::proto::SovHeader as RawSovHeader;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientMessage {
    Header(Box<SovHeader>),
    Misbehaviour(Box<SovMisbehaviour>),
}

impl Protobuf<Any> for ClientMessage {}

impl TryFrom<Any> for ClientMessage {
    type Error = ContractError;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        let msg = match &*any.type_url {
            SOVEREIGN_HEADER_TYPE_URL => {
                let header = Protobuf::<RawSovHeader>::decode(&*any.value)
                    .map_err(|e| ContractError::Celestia(format!("{e:?}")))?;
                Self::Header(Box::new(header))
            }
            SOVEREIGN_MISBEHAVIOUR_TYPE_URL => {
                let misbehaviour = Protobuf::<RawSovMisbehaviour>::decode(&*any.value)
                    .map_err(|e| ContractError::Celestia(format!("{e:?}")))?;
                Self::Misbehaviour(Box::new(misbehaviour))
            }
            _ => Err(ContractError::Celestia(format!(
                "Unknown type: {}",
                any.type_url
            )))?,
        };

        Ok(msg)
    }
}

impl From<ClientMessage> for Any {
    fn from(msg: ClientMessage) -> Self {
        match msg {
            ClientMessage::Header(header) => Any {
                type_url: SOVEREIGN_HEADER_TYPE_URL.to_string(),
                value: Protobuf::<Any>::encode_vec(&*header).unwrap(),
            },
            ClientMessage::Misbehaviour(misbehaviour) => Any {
                type_url: SOVEREIGN_MISBEHAVIOUR_TYPE_URL.to_string(),
                value: Protobuf::<Any>::encode_vec(&*misbehaviour).unwrap(),
            },
        }
    }
}
