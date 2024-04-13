//! Defines the misbehaviour type for the Sovereign light client

use alloc::format;
use core::fmt::Debug;

use ibc_client_tendermint::types::{Header as TmHeader, Misbehaviour as TmMisbehaviour};
use ibc_core::client::types::error::ClientError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::primitives::proto::{Any, Protobuf};

use super::header::{Header, SovTmHeader};
use crate::proto::v1::Misbehaviour as RawSovTmMisbehaviour;
use crate::sovereign::Error;

pub const SOV_TENDERMINT_MISBEHAVIOUR_TYPE_URL: &str =
    "/ibc.lightclients.sovereign.tendermint.v1.Misbehaviour";

/// Sovereign light client's misbehaviour type
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq, Eq)]
pub struct Misbehaviour<H: Clone> {
    client_id: ClientId,
    header_1: Box<Header<H>>,
    header_2: Box<Header<H>>,
}

impl<H: Clone> Misbehaviour<H> {
    /// Creates a new misbehaviour
    pub fn new(client_id: ClientId, header_1: Header<H>, header_2: Header<H>) -> Self {
        Self {
            client_id,
            header_1: Box::new(header_1),
            header_2: Box::new(header_2),
        }
    }

    /// Getter for the client identifier
    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    /// Getter for the first header
    pub fn header_1(&self) -> &Header<H> {
        &self.header_1
    }

    /// Getter for the second header
    pub fn header_2(&self) -> &Header<H> {
        &self.header_2
    }
}

impl<H: Clone> Debug for Misbehaviour<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Misbehaviour {{ client_id: {:?}, header_1: {{...}}, header_2: {{...}} }}",
            self.client_id,
        )
    }
}

/// Misbehaviour type alias for the Sovereign SDK rollups operating on the
/// Tendermint-driven DA layer.
pub type SovTmMisbehaviour = Misbehaviour<TmHeader>;

impl SovTmMisbehaviour {
    /// Protobuf decoding of the `SovTmMisbehaviour` through the `RawSovTmMisbehaviour` type.
    pub fn decode_thru_raw(value: Vec<u8>) -> Result<Self, Error> {
        Protobuf::<RawSovTmMisbehaviour>::decode(&mut value.as_slice()).map_err(Error::source)
    }

    pub fn validate_basic(&self) -> Result<(), Error> {
        self.header_1.validate_basic()?;
        self.header_2.validate_basic()?;

        if self.header_1.da_header.signed_header.header.chain_id
            != self.header_2.da_header.signed_header.header.chain_id
        {
            return Err(Error::invalid("headers must have identical chain_ids"));
        }

        if self.header_1.height() < self.header_2.height() {
            return Err(Error::invalid(format!(
                "header_1 height is less than header_2 height ({} < {})",
                self.header_1.height(),
                self.header_2.height()
            )));
        }

        Ok(())
    }

    pub fn into_tendermint_misbehaviour(&self) -> TmMisbehaviour {
        TmMisbehaviour::new(
            self.client_id.clone(),
            self.header_1.da_header.clone(),
            self.header_2.da_header.clone(),
        )
    }
}

impl core::fmt::Display for SovTmMisbehaviour {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "{} h1: {}-{} h2: {}-{}",
            self.client_id,
            self.header_1.height(),
            self.header_1.da_header,
            self.header_2.height(),
            self.header_2.da_header,
        )
    }
}

impl Protobuf<RawSovTmMisbehaviour> for SovTmMisbehaviour {}

impl TryFrom<RawSovTmMisbehaviour> for SovTmMisbehaviour {
    type Error = ClientError;
    #[allow(deprecated)]
    fn try_from(raw: RawSovTmMisbehaviour) -> Result<Self, Self::Error> {
        let client_id = raw.client_id.parse().map_err(|_| ClientError::Other {
            description: "".into(),
        })?;

        let header_1: SovTmHeader = raw
            .header_1
            .ok_or(ClientError::Other {
                description: "".into(),
            })?
            .try_into()?;

        let header_2: SovTmHeader = raw
            .header_2
            .ok_or(ClientError::Other {
                description: "".into(),
            })?
            .try_into()?;

        Ok(Self::new(client_id, header_1, header_2))
    }
}

impl From<SovTmMisbehaviour> for RawSovTmMisbehaviour {
    fn from(value: SovTmMisbehaviour) -> Self {
        #[allow(deprecated)]
        RawSovTmMisbehaviour {
            client_id: value.client_id.to_string(),
            header_1: Some((*value.header_1).into()),
            header_2: Some((*value.header_2).into()),
        }
    }
}

impl Protobuf<Any> for SovTmMisbehaviour {}

impl TryFrom<Any> for SovTmMisbehaviour {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        fn decode_misbehaviour(value: &[u8]) -> Result<SovTmMisbehaviour, ClientError> {
            let misbehaviour = Protobuf::<RawSovTmMisbehaviour>::decode(value).map_err(|e| {
                ClientError::Other {
                    description: e.to_string(),
                }
            })?;
            Ok(misbehaviour)
        }

        match raw.type_url.as_str() {
            SOV_TENDERMINT_MISBEHAVIOUR_TYPE_URL => decode_misbehaviour(&raw.value),
            _ => Err(ClientError::Other {
                description: "".into(),
            }),
        }
    }
}

impl From<SovTmMisbehaviour> for Any {
    fn from(misbehaviour: SovTmMisbehaviour) -> Self {
        Any {
            type_url: SOV_TENDERMINT_MISBEHAVIOUR_TYPE_URL.to_string(),
            value: Protobuf::<RawSovTmMisbehaviour>::encode_vec(misbehaviour),
        }
    }
}
