use core::time::Duration;

use ibc_client_tendermint::types::proto::v1::Fraction;
use ibc_client_tendermint::types::TrustThreshold;
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::primitives::proto::Protobuf;

use crate::proto::v1::TendermintClientParams as RawTmClientParams;
use crate::sovereign::Error;

/// Defines the Tendermint-specific client state parameters
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TendermintClientParams {
    pub chain_id: ChainId,
    pub trust_level: TrustThreshold,
    pub unbonding_period: Duration,
    pub max_clock_drift: Duration,
}

impl TendermintClientParams {
    pub fn new(
        chain_id: ChainId,
        trust_level: TrustThreshold,
        unbonding_period: Duration,
        max_clock_drift: Duration,
    ) -> Self {
        Self {
            chain_id,
            trust_level,
            unbonding_period,
            max_clock_drift,
        }
    }

    /// Returns `true` if the respective fields of two `TendermintClientParams`
    /// match for the client recovery process.
    pub fn check_on_recovery(&self, substitute: &Self) -> bool {
        self.trust_level == substitute.trust_level
            && self.unbonding_period == substitute.unbonding_period
            && self.max_clock_drift == substitute.max_clock_drift
    }

    /// Updates the `TendermintClientParams` on the client recovery process with
    /// the given substitute.
    pub fn update_on_recovery(self, substitute: Self) -> Self {
        Self {
            chain_id: substitute.chain_id,
            ..self
        }
    }

    /// Updates the respective fields of the `TendermintClientParams` on the
    /// client upgrade process with the given upgraded client parameters.
    pub fn update_on_upgrade(self, upgraded: Self) -> Self {
        Self {
            chain_id: upgraded.chain_id,
            unbonding_period: upgraded.unbonding_period,
            ..self
        }
    }
}

impl Protobuf<RawTmClientParams> for TendermintClientParams {}

impl TryFrom<RawTmClientParams> for TendermintClientParams {
    type Error = Error;

    fn try_from(raw: RawTmClientParams) -> Result<Self, Self::Error> {
        let chain_id = raw.chain_id.parse().map_err(Error::source)?;

        let trust_level = {
            let fraction = raw.trust_level.ok_or(Error::missing("trust_level"))?;
            TrustThreshold::new(fraction.numerator, fraction.denominator)?
        };

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
            unbonding_period,
            max_clock_drift,
        ))
    }
}

impl From<TendermintClientParams> for RawTmClientParams {
    fn from(value: TendermintClientParams) -> Self {
        Self {
            chain_id: value.chain_id.to_string(),
            trust_level: Some(Fraction {
                numerator: value.trust_level.numerator(),
                denominator: value.trust_level.denominator(),
            }),
            unbonding_period: Some(value.unbonding_period.into()),
            max_clock_drift: Some(value.max_clock_drift.into()),
        }
    }
}
