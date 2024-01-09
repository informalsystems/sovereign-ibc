//! Defines the misbehaviour type for the Sovereign light client

use alloc::format;

use ibc_client_tendermint::types::{Header as TmHeader, Misbehaviour};
use ibc_core::client::types::error::ClientError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::primitives::proto::Any;
use ibc_proto::ibc::lightclients::sovereign::tendermint::v1::SovTendermintMisbehaviour as RawSovTmMisbehaviour;
use tendermint_proto::Protobuf;

use super::sov_header::{SovHeader, SovTmHeader};
use crate::error::Error;

pub const SOV_TENDERMINT_MISBEHAVIOUR_TYPE_URL: &str =
    "/ibc.lightclients.sovereign.tendermint.v1.Misbehaviour";

/// Sovereign light client's misbehaviour type
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SovMisbehaviour<H: Clone> {
    client_id: ClientId,
    header1: SovHeader<H>,
    header2: SovHeader<H>,
}

/// Misbehaviour type alias for the Sovereign SDK rollups operating on the
/// Tendermint-driven DA layer.
pub type SovTmMisbehaviour = SovMisbehaviour<TmHeader>;

impl SovTmMisbehaviour {
    pub fn new(client_id: ClientId, header1: SovTmHeader, header2: SovTmHeader) -> Self {
        Self {
            client_id,
            header1,
            header2,
        }
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    pub fn header1(&self) -> &SovTmHeader {
        &self.header1
    }

    pub fn header2(&self) -> &SovTmHeader {
        &self.header2
    }

    pub fn validate_basic(&self) -> Result<(), Error> {
        self.header1.validate_basic()?;
        self.header2.validate_basic()?;

        if self.header1.da_header.signed_header.header.chain_id
            != self.header2.da_header.signed_header.header.chain_id
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

    pub fn into_tendermint_misbehaviour(&self) -> Misbehaviour {
        Misbehaviour::new(
            self.client_id.clone(),
            self.header1.da_header.clone(),
            self.header2.da_header.clone(),
        )
    }
}

impl core::fmt::Display for SovTmMisbehaviour {
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

impl Protobuf<RawSovTmMisbehaviour> for SovTmMisbehaviour {}

impl TryFrom<RawSovTmMisbehaviour> for SovTmMisbehaviour {
    type Error = ClientError;
    #[allow(deprecated)]
    fn try_from(raw: RawSovTmMisbehaviour) -> Result<Self, Self::Error> {
        let client_id = raw.client_id.parse().map_err(|_| ClientError::Other {
            description: "".into(),
        })?;

        let header1: SovTmHeader = raw
            .header_1
            .ok_or(ClientError::Other {
                description: "".into(),
            })?
            .try_into()?;

        let header2: SovTmHeader = raw
            .header_2
            .ok_or(ClientError::Other {
                description: "".into(),
            })?
            .try_into()?;

        Ok(Self::new(client_id, header1, header2))
    }
}

impl From<SovTmMisbehaviour> for RawSovTmMisbehaviour {
    fn from(value: SovTmMisbehaviour) -> Self {
        #[allow(deprecated)]
        RawSovTmMisbehaviour {
            client_id: value.client_id.to_string(),
            header_1: Some(value.header1.into()),
            header_2: Some(value.header2.into()),
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
