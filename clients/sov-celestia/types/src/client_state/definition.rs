use core::cmp::max;
use std::str::FromStr;

use ibc_client_tendermint::types::{Header as TmHeader, TrustThreshold};
use ibc_core::client::types::error::{ClientError, UpgradeClientError};
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::CommitmentPrefix;
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_core::primitives::ZERO_DURATION;
use tendermint_light_client_verifier::options::Options;

use super::TmClientParams;
use crate::error::Error;
use crate::proto::tendermint::v1::ClientState as RawClientState;

pub const SOV_TENDERMINT_CLIENT_STATE_TYPE_URL: &str =
    "/ibc.lightclients.sovereign.tendermint.v1.ClientState";

/// Contains the core implementation of the Sovereign light client
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClientState<Da> {
    pub rollup_id: ChainId,
    pub latest_height: Height,
    pub frozen_height: Option<Height>,
    pub upgrade_path: UpgradePath,
    pub da_params: Da,
}

impl<Da> ClientState<Da> {
    pub fn new(
        rollup_id: ChainId,
        latest_height: Height,
        frozen_height: Option<Height>,
        upgrade_path: UpgradePath,
        da_params: Da,
    ) -> Self {
        Self {
            rollup_id,
            latest_height,
            frozen_height,
            upgrade_path,
            da_params,
        }
    }

    pub fn rollup_id(&self) -> &ChainId {
        &self.rollup_id
    }

    pub fn latest_height(&self) -> Height {
        self.latest_height
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen_height.is_some()
    }

    pub fn with_frozen_height(self, h: Height) -> Self {
        Self {
            frozen_height: Some(h),
            ..self
        }
    }
}

pub type SovTmClientState = ClientState<TmClientParams>;

impl SovTmClientState {
    pub fn chain_id(&self) -> &ChainId {
        &self.da_params.chain_id
    }

    pub fn with_header(self, header: TmHeader) -> Result<Self, Error> {
        Ok(Self {
            latest_height: max(header.height(), self.latest_height),
            ..self
        })
    }

    /// Helper method to produce a [`Options`] struct for use in
    /// Tendermint-specific light client verification.
    pub fn as_light_client_options(&self) -> Result<Options, Error> {
        Ok(Options {
            trust_threshold: self
                .da_params
                .trust_level
                .try_into()
                .map_err(Error::source)?,
            trusting_period: self.da_params.trusting_period,
            clock_drift: self.da_params.max_clock_drift,
        })
    }

    // Resets custom fields to zero values (used in `update_client`)
    pub fn zero_custom_fields(&mut self) {
        self.frozen_height = None;
        self.da_params.trusting_period = ZERO_DURATION;
        self.da_params.trust_level = TrustThreshold::ZERO;
        self.da_params.max_clock_drift = ZERO_DURATION;
    }
}

impl Protobuf<RawClientState> for SovTmClientState {}

impl TryFrom<RawClientState> for SovTmClientState {
    type Error = ClientError;

    fn try_from(raw: RawClientState) -> Result<Self, Self::Error> {
        let rollup_id = raw.rollup_id.parse().map_err(Error::source)?;

        let latest_height = raw
            .latest_height
            .ok_or(Error::missing("latest_height"))?
            .try_into()?;

        let upgrade_path = raw.upgrade_path;

        let tendermint_params = raw
            .tendermint_params
            .ok_or(Error::missing("tendermint_params"))?
            .try_into()?;

        Ok(Self::new(
            rollup_id,
            latest_height,
            raw.frozen_height.map(TryInto::try_into).transpose()?,
            upgrade_path.try_into()?,
            tendermint_params,
        ))
    }
}

impl From<SovTmClientState> for RawClientState {
    fn from(value: SovTmClientState) -> Self {
        Self {
            rollup_id: value.rollup_id.to_string(),
            latest_height: Some(value.latest_height.into()),
            frozen_height: value.frozen_height.map(|h| h.into()),
            upgrade_path: value.upgrade_path.0,
            tendermint_params: Some(value.da_params.into()),
        }
    }
}

impl Protobuf<Any> for SovTmClientState {}

impl TryFrom<Any> for SovTmClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        fn decode_client_state(value: &[u8]) -> Result<SovTmClientState, ClientError> {
            let client_state =
                Protobuf::<RawClientState>::decode(value).map_err(|e| ClientError::Other {
                    description: e.to_string(),
                })?;

            Ok(client_state)
        }

        match raw.type_url.as_str() {
            SOV_TENDERMINT_CLIENT_STATE_TYPE_URL => decode_client_state(&raw.value),
            _ => Err(ClientError::UnknownClientStateType {
                client_state_type: raw.type_url,
            }),
        }
    }
}

impl From<SovTmClientState> for Any {
    fn from(client_state: SovTmClientState) -> Self {
        Any {
            type_url: SOV_TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawClientState>::encode_vec(client_state),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct UpgradePath(String);

impl Default for UpgradePath {
    fn default() -> Self {
        Self("sov_ibc/Ibc/".to_string())
    }
}

impl UpgradePath {
    pub fn new(path: String) -> Self {
        Self(path)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for UpgradePath {
    type Error = ClientError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl FromStr for UpgradePath {
    type Err = ClientError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(UpgradeClientError::Other {
                reason: "empty upgrade path".into(),
            })?;
        }

        Ok(Self(s.to_string()))
    }
}

impl TryFrom<UpgradePath> for CommitmentPrefix {
    type Error = ClientError;

    fn try_from(value: UpgradePath) -> Result<Self, Self::Error> {
        CommitmentPrefix::try_from(value.0.into_bytes())
            .map_err(ClientError::InvalidCommitmentProof)
    }
}
