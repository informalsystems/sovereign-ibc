use core::fmt::{Display, Error as FmtError, Formatter};

use ibc_client_tendermint::types::Header as TmHeader;
use ibc_core::client::types::Height;
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::primitives::proto::Any;
use ibc_core::primitives::Timestamp;
use tendermint_proto::Protobuf;

use super::aggregated_proof::AggregatedProof;
use crate::client_message::pretty::PrettyAggregatedProof;
use crate::error::Error;
use crate::proto::SovTendermintHeader as RawSovTmHeader;

pub const SOV_TENDERMINT_HEADER_TYPE_URL: &str = "/ibc.lightclients.sovereign.tendermint.v1.Header";

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SovHeader<H: Clone> {
    pub core_header: H,
    pub aggregated_proof: AggregatedProof,
}

impl<H: Clone> core::fmt::Debug for SovHeader<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, " SovHeader {{...}}")
    }
}

impl<H: Clone + Display> Display for SovHeader<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "SovHeader {{ core_header: {}, aggregated_proof: {} }}",
            &self.core_header,
            PrettyAggregatedProof(&self.aggregated_proof)
        )
    }
}

/// Header type alias for the Sovereign SDK rollups operating on the
/// Tendermint-driven DA layer.
pub type SovTmHeader = SovHeader<TmHeader>;

impl SovTmHeader {
    pub fn timestamp(&self) -> Timestamp {
        self.core_header.timestamp()
    }

    pub fn height(&self) -> Height {
        self.core_header.height()
    }

    pub fn verify_chain_id_version_matches_height(&self, chain_id: &ChainId) -> Result<(), Error> {
        self.core_header
            .verify_chain_id_version_matches_height(chain_id)
            .map_err(Error::source)
    }

    /// Checks if the fields of a given header are consistent with the trusted fields of this header.
    pub fn validate_basic(&self) -> Result<(), Error> {
        self.core_header.validate_basic().map_err(Error::source)
    }
}

impl Protobuf<RawSovTmHeader> for SovTmHeader {}

impl TryFrom<RawSovTmHeader> for SovTmHeader {
    type Error = Error;

    fn try_from(value: RawSovTmHeader) -> Result<Self, Self::Error> {
        let core_header = value
            .core_header
            .ok_or(Error::missing("missing core header"))?;

        let core_header = TmHeader::try_from(core_header).map_err(Error::source)?;

        let aggregate_snark = value
            .aggregated_proof
            .ok_or(Error::missing("missing aggregate_snark"))?
            .try_into()
            .map_err(Error::source)?;

        Ok(SovHeader {
            core_header,
            aggregated_proof: aggregate_snark,
        })
    }
}

impl From<SovTmHeader> for RawSovTmHeader {
    fn from(value: SovTmHeader) -> RawSovTmHeader {
        RawSovTmHeader {
            core_header: Some(value.core_header.into()),
            aggregated_proof: Some(value.aggregated_proof.into()),
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
