use core::fmt::{Display, Error as FmtError, Formatter};

use celestia_types::ExtendedHeader;
use ibc::Height;
use tendermint::validator;
use tendermint_proto::Protobuf;

use crate::types::client_message::pretty::{PrettyExtendedHeader, PrettyValidatorSet};
use crate::types::error::Error;
use crate::types::proto::CelestiaHeader as RawCelestiaHeader;

pub type ValidatorSet = validator::Set;

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CelestiaHeader {
    pub extended_header: ExtendedHeader,
    pub trusted_height: Height,
    pub trusted_next_validator_set: ValidatorSet,
}

impl Protobuf<RawCelestiaHeader> for CelestiaHeader {}

impl TryFrom<RawCelestiaHeader> for CelestiaHeader {
    type Error = Error;

    fn try_from(value: RawCelestiaHeader) -> Result<Self, Self::Error> {
        let extended_header = value
            .extended_header
            .ok_or(Error::missing("missing signed_header"))?;

        let extended_header = ExtendedHeader::try_from(extended_header).map_err(Error::source)?;

        let trusted_height = value
            .trusted_height
            .ok_or(Error::missing("missing trusted_height"))?
            .try_into()
            .map_err(Error::source)?;

        let trusted_next_validator_set = value
            .trusted_next_validator_set
            .ok_or(Error::missing("missing trusted_next_validator_set"))?
            .try_into()
            .map_err(Error::source)?;

        Ok(CelestiaHeader {
            extended_header,
            trusted_height,
            trusted_next_validator_set,
        })
    }
}

impl From<CelestiaHeader> for RawCelestiaHeader {
    fn from(value: CelestiaHeader) -> RawCelestiaHeader {
        RawCelestiaHeader {
            extended_header: Some(value.extended_header.into()),
            trusted_height: Some(value.trusted_height.into()),
            trusted_next_validator_set: Some(value.trusted_next_validator_set.into()),
        }
    }
}

impl core::fmt::Debug for CelestiaHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, " CelestiaHeader {{...}}")
    }
}

impl Display for CelestiaHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "Header {{ extended_header: {}, trusted_height: {}, trusted_next_validator_set: {} }}",
            PrettyExtendedHeader(&self.extended_header),
            self.trusted_height,
            PrettyValidatorSet(&self.trusted_next_validator_set)
        )
    }
}
