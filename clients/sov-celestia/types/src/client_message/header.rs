use core::fmt::{Debug, Display, Error as FmtError, Formatter};

use ibc_client_tendermint::types::Header as TmHeader;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_core::primitives::Timestamp;
use tendermint::chain::Id as TmChainId;
use tendermint_light_client_verifier::types::TrustedBlockState;

use super::aggregated_proof::AggregatedProof;
use crate::consensus_state::SovTmConsensusState;
use crate::error::Error;
use crate::proto::tendermint::v1::Header as RawSovTmHeader;

pub const SOV_TENDERMINT_HEADER_TYPE_URL: &str = "/ibc.lightclients.sovereign.tendermint.v1.Header";

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header<H: Clone> {
    pub da_header: H,
    pub aggregated_proof: AggregatedProof,
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
            "Header {{ da_header: {}, aggregated_proof: {} }}",
            &self.da_header, &self.aggregated_proof
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

    /// Returns the height of the Sovereign-Tendermint header.
    pub fn height(&self) -> Height {
        self.aggregated_proof.public_data.final_slot_number()
    }

    /// Returns the trusted height of the Sovereign-Tendermint header, which
    /// corresponds to the `trusted_height` field or the DA header.
    pub fn trusted_height(&self) -> Height {
        self.da_header.trusted_height
    }

    /// Performs sanity checks and validate if the fields of the given header
    /// are consistent with the trusted fields of this header.
    pub fn validate_basic(&self) -> Result<(), Error> {
        self.da_header.validate_basic().map_err(Error::source)?;

        self.aggregated_proof.validate_basic()?;

        if self.height() != self.da_header.height() {
            return Err(Error::mismatch(format!(
                "DA header height {} does not match aggregated proof height(rollup slot number) {}",
                self.da_header.height(),
                self.height()
            )))?;
        };

        Ok(())
    }

    /// Transforms the header into a `TrustedBlockState` which can be used for
    /// the DA header misbehaviour verification.
    pub fn as_trusted_da_block_state<'a>(
        &'a self,
        consensus_state: &SovTmConsensusState,
        chain_id: &'a TmChainId,
    ) -> Result<TrustedBlockState<'a>, Error> {
        Ok(TrustedBlockState {
            chain_id,
            header_time: consensus_state.timestamp(),
            height: self
                .trusted_height()
                .revision_height()
                .try_into()
                .map_err(Error::source)?,
            next_validators: &self.da_header.trusted_next_validator_set,
            next_validators_hash: consensus_state.da_params.next_validators_hash,
        })
    }
}

impl Protobuf<RawSovTmHeader> for SovTmHeader {}

impl TryFrom<RawSovTmHeader> for SovTmHeader {
    type Error = ClientError;

    fn try_from(value: RawSovTmHeader) -> Result<Self, Self::Error> {
        let raw_da_header = value
            .tendermint_header
            .ok_or(Error::missing("missing core header"))?;

        let da_header = TmHeader::try_from(raw_da_header).map_err(Error::source)?;

        let aggregated_proof = value
            .aggregated_proof
            .ok_or(Error::missing("missing aggregated proof"))?
            .try_into()?;

        Ok(Header {
            da_header,
            aggregated_proof,
        })
    }
}

impl From<SovTmHeader> for RawSovTmHeader {
    fn from(value: SovTmHeader) -> RawSovTmHeader {
        RawSovTmHeader {
            tendermint_header: Some(value.da_header.into()),
            aggregated_proof: Some(value.aggregated_proof.into()),
        }
    }
}

impl Protobuf<Any> for SovTmHeader {}

impl TryFrom<Any> for SovTmHeader {
    type Error = ClientError;

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
