use alloc::format;
use core::fmt::{Display, Error as FmtError, Formatter};
use core::str::FromStr;

use ibc::core::ics24_host::identifier::ChainId;
use ibc::core::timestamp::Timestamp;
use ibc::proto::Any;
use ibc::Height;
use tendermint_proto::Protobuf;

use super::aggregate_snark::AggregateSNARK;
use super::celestia_header::CelestiaHeader;
use crate::types::client_message::pretty::{PrettyAggregateSNARK, PrettyCelestiaHeader};
use crate::types::error::Error;
use crate::types::proto::SovHeader as RawSovHeader;

pub const SOVEREIGN_HEADER_TYPE_URL: &str = "/ibc.lightclients.sovereign.v1.Header";

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SovHeader {
    pub da_header: CelestiaHeader,
    pub aggregate_snark: AggregateSNARK,
}

impl Protobuf<RawSovHeader> for SovHeader {}

impl TryFrom<RawSovHeader> for SovHeader {
    type Error = Error;

    fn try_from(value: RawSovHeader) -> Result<Self, Self::Error> {
        let da_header = value.da_header.ok_or(Error::missing("missing da_header"))?;

        let da_header = CelestiaHeader::try_from(da_header).map_err(Error::source)?;

        let aggregate_snark = value
            .aggregate_snark
            .ok_or(Error::missing("missing aggregate_snark"))?
            .try_into()
            .map_err(Error::source)?;

        Ok(SovHeader {
            da_header,
            aggregate_snark,
        })
    }
}

impl From<SovHeader> for RawSovHeader {
    fn from(value: SovHeader) -> RawSovHeader {
        RawSovHeader {
            da_header: Some(value.da_header.into()),
            aggregate_snark: Some(value.aggregate_snark.into()),
        }
    }
}

impl Protobuf<Any> for SovHeader {}

impl TryFrom<Any> for SovHeader {
    type Error = Error;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        let msg = match &*any.type_url {
            SOVEREIGN_HEADER_TYPE_URL => {
                Protobuf::<RawSovHeader>::decode_vec(&any.value).map_err(Error::source)?
            }
            _ => Err(Error::invalid(any.type_url.clone()))?,
        };

        Ok(msg)
    }
}

impl From<SovHeader> for Any {
    fn from(msg: SovHeader) -> Self {
        Any {
            type_url: SOVEREIGN_HEADER_TYPE_URL.to_string(),
            value: Protobuf::<Any>::encode_vec(&msg).unwrap(),
        }
    }
}

impl core::fmt::Debug for SovHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, " SovHeader {{...}}")
    }
}

impl Display for SovHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "Header {{ da_header: {}, aggregate_snark: {} }}",
            PrettyCelestiaHeader(&self.da_header),
            PrettyAggregateSNARK(&self.aggregate_snark)
        )
    }
}

impl SovHeader {
    pub fn timestamp(&self) -> Timestamp {
        let time = self
            .da_header
            .extended_header
            .header
            .time
            .unix_timestamp_nanos();

        Timestamp::from_nanoseconds(time as u64).expect("malformed header")
    }

    pub fn height(&self) -> Height {
        Height::new(
            ChainId::from_str(self.da_header.extended_header.header.chain_id.as_str())
                .expect("chain id")
                .revision_number(),
            u64::from(self.da_header.extended_header.header.height),
        )
        .expect("malformed Sovereign header domain type has an illegal height of 0")
    }

    pub fn verify_chain_id_version_matches_height(&self, chain_id: &ChainId) -> Result<(), Error> {
        if self.height().revision_number() != chain_id.revision_number() {
            return Err(Error::mismatch(format!(
                "chain id revision number does not match header revision number (chain id revision number: {}, header revision number: {})",
                chain_id.revision_number(),
                self.height().revision_number()
            )));
        }
        Ok(())
    }

    /// Checks if the fields of a given header are consistent with the trusted fields of this header.
    pub fn validate_basic(&self) -> Result<(), Error> {
        if self.height().revision_number() != self.da_header.trusted_height.revision_number() {
            return Err(Error::mismatch(format!(
                "header height revision number does not match trusted height revision number (header height revision number: {}, trusted height revision number: {})",
                self.height().revision_number(),
                self.da_header.trusted_height.revision_number()
            )));
        }

        // We need to ensure that the trusted height (representing the
        // height of the header already on chain for which this client update is
        // based on) must be smaller than height of the new header that we're
        // installing.
        if self.da_header.trusted_height >= self.height() {
            return Err(Error::invalid(format!(
                "header height is not greater than trusted height (header height: {}, trusted height: {})",
                self.height(),
                self.da_header.trusted_height
            )));
        }

        if self.da_header.extended_header.validator_set.hash()
            != self.da_header.extended_header.header.validators_hash
        {
            return Err(Error::mismatch(format!(
                "header validator set hash does not match trusted validator set hash (header validator set hash: {}, trusted validator set hash: {})",
                self.da_header.extended_header.validator_set.hash(),
                self.da_header.extended_header.header.validators_hash
            )));
        }

        Ok(())
    }
}
