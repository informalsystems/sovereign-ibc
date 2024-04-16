use core::fmt::{Debug, Display, Error as FmtError, Formatter};

use ibc_client_tendermint::types::Header as TmHeader;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_core::primitives::Timestamp;
use tendermint::chain::Id as TmChainId;
use tendermint::crypto::default::Sha256;
use tendermint_light_client_verifier::types::TrustedBlockState;

use crate::consensus_state::SovTmConsensusState;
use crate::proto::v1::Header as RawSovTmHeader;
use crate::sovereign::{AggregatedProof, Error};

pub const SOV_TENDERMINT_HEADER_TYPE_URL: &str = "/ibc.lightclients.sovereign.tendermint.v1.Header";

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header<H: Clone> {
    pub aggregated_proof: AggregatedProof,
    pub da_header: H,
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
            "Header {{ aggregated_proof: {}, da_header: {} }}",
            &self.aggregated_proof, &self.da_header
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
        Height::new(0, self.aggregated_proof.final_slot_number())
            .expect("zero slot number rejected beforehand")
    }

    /// Returns the trusted height of the Sovereign-Tendermint header, which
    /// corresponds to the `trusted_height` field or the DA header.
    pub fn trusted_height(&self) -> Height {
        self.da_header.trusted_height
    }

    /// Performs sanity checks on header to ensure the consistency of fields.
    pub fn validate_basic(&self) -> Result<(), Error> {
        self.da_header
            .validate_basic::<Sha256>()
            .map_err(Error::source)?;

        self.aggregated_proof.validate_basic()?;

        Ok(())
    }

    /// Validates the height offset between the rollup and DA layer.
    pub fn validate_da_height_offset(
        &self,
        genesis_da_height: Height,
        client_latest_height: Height,
    ) -> Result<(), ClientError> {
        let expected_da_height = self.height().add(genesis_da_height.revision_height());

        let given_da_height = self.da_header.height();

        if expected_da_height != given_da_height {
            return Err(ClientError::Other {
                description: format!(
                    "The height of the DA header does not match expected height:\
                    got '{given_da_height}', expected '{expected_da_height}'",
                ),
            });
        }

        let client_height_in_da = client_latest_height.add(genesis_da_height.revision_height());

        let header_trusted_height = self.da_header.trusted_height;

        if client_height_in_da != header_trusted_height {
            return Err(ClientError::Other {
                description: format!(
                    "trusted DA height does not match expected height:\
                    got {header_trusted_height}, expected {client_height_in_da}",
                ),
            });
        };
        Ok(())
    }

    /// Transforms the header into a `TrustedBlockState`, used for the DA header
    /// misbehaviour verification.
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
