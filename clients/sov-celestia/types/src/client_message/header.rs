use core::fmt::{Debug, Display, Error as FmtError, Formatter};

use ibc_client_tendermint::types::Header as TmHeader;
use ibc_core::client::types::Height;
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_core::primitives::Timestamp;
use ibc_proto::ibc::lightclients::sovereign::tendermint::v1::Header as RawSovTmHeader;
use prost::Message;

use super::aggregated_proof::AggregatedProofData;
use crate::client_message::pretty::PrettyAggregatedProofData;
use crate::error::Error;

pub const SOV_TENDERMINT_HEADER_TYPE_URL: &str = "/ibc.lightclients.sovereign.tendermint.v1.Header";

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header<H: Clone> {
    pub da_header: H,
    pub aggregated_proof_data: AggregatedProofData,
}

impl<H: Clone> Debug for Header<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "Header {{...}}")
    }
}

impl<H: Clone + Display> Display for Header<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "Header {{ da_header: {}, aggregated_proof_data: {} }}",
            &self.da_header,
            PrettyAggregatedProofData(&self.aggregated_proof_data)
        )
    }
}

/// Header type alias for the Sovereign SDK rollups operating on the
/// Tendermint-driven DA layer.
pub type SovTmHeader = Header<TmHeader>;

impl SovTmHeader {
    pub fn timestamp(&self) -> Timestamp {
        self.da_header.timestamp()
    }

    pub fn height(&self) -> Height {
        self.da_header.height()
    }

    /// Protobuf encoding of the `SovTmHeader` through the `Any` type.
    pub fn encode_thru_any(self) -> Vec<u8> {
        Any::from(self).encode_to_vec()
    }

    /// Protobuf decoding of the `SovTmHeader` through the `RawSovTmHeader` type.
    pub fn decode_thru_raw(value: Vec<u8>) -> Result<Self, Error> {
        Protobuf::<RawSovTmHeader>::decode(&mut value.as_slice()).map_err(Error::source)
    }

    pub fn verify_chain_id_version_matches_height(&self, chain_id: &ChainId) -> Result<(), Error> {
        self.da_header
            .verify_chain_id_version_matches_height(chain_id)
            .map_err(Error::source)
    }

    /// Checks if the fields of a given header are consistent with the trusted fields of this header.
    pub fn validate_basic(&self) -> Result<(), Error> {
        self.da_header.validate_basic().map_err(Error::source)
    }
}

impl Protobuf<RawSovTmHeader> for SovTmHeader {}

impl TryFrom<RawSovTmHeader> for SovTmHeader {
    type Error = Error;

    fn try_from(value: RawSovTmHeader) -> Result<Self, Self::Error> {
        let da_header = value
            .tendermint_header
            .ok_or(Error::missing("missing core header"))?;

        let da_header = TmHeader::try_from(da_header).map_err(Error::source)?;

        let aggregated_proof_data = value
            .aggregated_proof_data
            .ok_or(Error::missing("missing aggregated proof"))?
            .try_into()?;

        Ok(Header {
            da_header,
            aggregated_proof_data,
        })
    }
}

impl From<SovTmHeader> for RawSovTmHeader {
    fn from(value: SovTmHeader) -> RawSovTmHeader {
        RawSovTmHeader {
            tendermint_header: Some(value.da_header.into()),
            aggregated_proof_data: Some(value.aggregated_proof_data.into()),
        }
    }
}

impl Protobuf<Any> for SovTmHeader {}

impl TryFrom<Any> for SovTmHeader {
    type Error = Error;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        let msg = match any.type_url.as_str() {
            SOV_TENDERMINT_HEADER_TYPE_URL => {
                Protobuf::<RawSovTmHeader>::decode_vec(&any.value).map_err(Error::source)?
            }
            _ => Err(Error::invalid(any.type_url.clone()))?,
        };

        Ok(msg)
    }
}

impl From<SovTmHeader> for Any {
    fn from(header: SovTmHeader) -> Self {
        Any {
            type_url: SOV_TENDERMINT_HEADER_TYPE_URL.to_string(),
            value: Protobuf::<RawSovTmHeader>::encode_vec(header),
        }
    }
}
