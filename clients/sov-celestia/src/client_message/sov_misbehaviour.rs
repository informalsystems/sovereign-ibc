//! Defines the misbehaviour type for the Sovereign light client

use alloc::format;
use alloc::string::String;

use ibc_core::client::types::error::ClientError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::primitives::proto::Any;
use tendermint_proto::Protobuf;

use super::sov_header::SovHeader;
use crate::error::Error;
use crate::proto::SovHeader as RawSovHeader;

pub const SOVEREIGN_MISBEHAVIOUR_TYPE_URL: &str = "/ibc.lightclients.sovereign.v1.Misbehaviour";

/// Sovereign light client's misbehaviour type
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SovMisbehaviour {
    client_id: ClientId,
    header1: SovHeader,
    header2: SovHeader,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct RawSovMisbehaviour {
    #[prost(string, tag = "1")]
    pub client_id: String,
    #[prost(message, optional, tag = "2")]
    pub header_1: Option<RawSovHeader>,
    #[prost(message, optional, tag = "3")]
    pub header_2: Option<RawSovHeader>,
}

impl Protobuf<RawSovMisbehaviour> for SovMisbehaviour {}

impl TryFrom<RawSovMisbehaviour> for SovMisbehaviour {
    type Error = ClientError;
    #[allow(deprecated)]
    fn try_from(raw: RawSovMisbehaviour) -> Result<Self, Self::Error> {
        let client_id = raw.client_id.parse().map_err(|_| ClientError::Other {
            description: "".into(),
        })?;

        let header1: SovHeader = raw
            .header_1
            .ok_or(ClientError::Other {
                description: "".into(),
            })?
            .try_into()?;

        let header2: SovHeader = raw
            .header_2
            .ok_or(ClientError::Other {
                description: "".into(),
            })?
            .try_into()?;

        Ok(Self::new(client_id, header1, header2))
    }
}

impl From<SovMisbehaviour> for RawSovMisbehaviour {
    fn from(value: SovMisbehaviour) -> Self {
        #[allow(deprecated)]
        RawSovMisbehaviour {
            client_id: value.client_id.to_string(),
            header_1: Some(value.header1.into()),
            header_2: Some(value.header2.into()),
        }
    }
}

impl Protobuf<Any> for SovMisbehaviour {}

impl TryFrom<Any> for SovMisbehaviour {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        fn decode_misbehaviour(value: &[u8]) -> Result<SovMisbehaviour, ClientError> {
            let misbehaviour =
                Protobuf::<RawSovMisbehaviour>::decode(value).map_err(|e| ClientError::Other {
                    description: e.to_string(),
                })?;
            Ok(misbehaviour)
        }

        match raw.type_url.as_str() {
            SOVEREIGN_MISBEHAVIOUR_TYPE_URL => decode_misbehaviour(&raw.value),
            _ => Err(ClientError::Other {
                description: "".into(),
            }),
        }
    }
}

impl From<SovMisbehaviour> for Any {
    fn from(misbehaviour: SovMisbehaviour) -> Self {
        Any {
            type_url: SOVEREIGN_MISBEHAVIOUR_TYPE_URL.to_string(),
            value: Protobuf::<RawSovMisbehaviour>::encode_vec(&misbehaviour).unwrap(),
        }
    }
}

impl core::fmt::Display for SovMisbehaviour {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "{} h1: {}-{} h2: {}-{}",
            self.client_id,
            self.header1.height(),
            self.header1.da_header,
            self.header2.height(),
            self.header2.da_header,
        )
    }
}

impl SovMisbehaviour {
    pub fn new(client_id: ClientId, header1: SovHeader, header2: SovHeader) -> Self {
        Self {
            client_id,
            header1,
            header2,
        }
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    pub fn header1(&self) -> &SovHeader {
        &self.header1
    }

    pub fn header2(&self) -> &SovHeader {
        &self.header2
    }

    pub fn validate_basic(&self) -> Result<(), Error> {
        self.header1.validate_basic()?;
        self.header2.validate_basic()?;

        if self.header1.da_header.extended_header.header.chain_id
            != self.header2.da_header.extended_header.header.chain_id
        {
            return Err(Error::invalid("headers must have identical chain_ids"));
        }

        if self.header1.height() < self.header2.height() {
            return Err(Error::invalid(format!(
                "header1 height is less than header2 height ({} < {})",
                self.header1.height(),
                self.header2.height()
            )));
        }

        Ok(())
    }
}
