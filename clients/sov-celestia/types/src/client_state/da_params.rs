use core::time::Duration;

use ibc_client_tendermint::types::TrustThreshold;
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::primitives::proto::Protobuf;
use ibc_proto::ibc::lightclients::sovereign::tendermint::v1::TendermintClientParams as RawTmClientParams;
use ibc_proto::ibc::lightclients::tendermint::v1::Fraction;

use crate::error::Error;

/// Defines the Tendermint-specific client state parameters
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TmClientParams {
    pub chain_id: ChainId,
    pub trust_level: TrustThreshold,
    pub trusting_period: Duration,
    pub unbonding_period: Duration,
    pub max_clock_drift: Duration,
}

impl TmClientParams {
    pub fn new(
        chain_id: ChainId,
        trust_level: TrustThreshold,
        trusting_period: Duration,
        unbonding_period: Duration,
        max_clock_drift: Duration,
    ) -> Self {
        Self {
            chain_id,
            trust_level,
            trusting_period,
            unbonding_period,
            max_clock_drift,
        }
    }
}

impl Protobuf<RawTmClientParams> for TmClientParams {}

impl TryFrom<RawTmClientParams> for TmClientParams {
    type Error = Error;

    fn try_from(raw: RawTmClientParams) -> Result<Self, Self::Error> {
        let chain_id = raw.chain_id.parse().map_err(Error::source)?;

        let trust_level = {
            let fraction = raw.trust_level.ok_or(Error::missing("trust_level"))?;
            TrustThreshold::new(fraction.numerator, fraction.denominator)?
        };

        let trusting_period = raw
            .trusting_period
            .ok_or(Error::missing("trusting_period"))?
            .try_into()
            .map_err(|_| Error::invalid("trusting_period"))?;

        let unbonding_period = raw
            .unbonding_period
            .ok_or(Error::missing("unbonding_period"))?
            .try_into()
            .map_err(|_| Error::invalid("unbonding_period"))?;

        let max_clock_drift = raw
            .max_clock_drift
            .ok_or(Error::missing("max_clock_drift"))?
            .try_into()
            .map_err(|_| Error::invalid("max_clock_drift"))?;

        Ok(Self::new(
            chain_id,
            trust_level,
            trusting_period,
            unbonding_period,
            max_clock_drift,
        ))
    }
}

impl From<TmClientParams> for RawTmClientParams {
    fn from(value: TmClientParams) -> Self {
        Self {
            chain_id: value.chain_id.to_string(),
            trust_level: Some(Fraction {
                numerator: value.trust_level.numerator(),
                denominator: value.trust_level.denominator(),
            }),
            trusting_period: Some(value.trusting_period.into()),
            unbonding_period: Some(value.unbonding_period.into()),
            max_clock_drift: Some(value.max_clock_drift.into()),
        }
    }
}
