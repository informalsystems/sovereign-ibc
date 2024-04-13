use core::cmp::max;
use core::time::Duration;

use ibc_client_tendermint::types::{Header as TmHeader, TrustThreshold};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_core::primitives::ZERO_DURATION;
use tendermint_light_client_verifier::options::Options;

use super::TendermintClientParams;
use crate::proto::v1::ClientState as RawClientState;
use crate::sovereign::{CodeCommitment, Error, Root, SovereignClientParams, UpgradePath};

pub const SOV_TENDERMINT_CLIENT_STATE_TYPE_URL: &str =
    "/ibc.lightclients.sovereign.tendermint.v1.ClientState";

/// Defines the `ClientState` type for the Sovereign SDK rollups.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClientState<Da> {
    pub sovereign_params: SovereignClientParams,
    pub da_params: Da,
}

impl<Da> ClientState<Da> {
    pub fn new(sovereign_params: SovereignClientParams, da_params: Da) -> Self {
        Self {
            sovereign_params,
            da_params,
        }
    }

    pub fn genesis_state_root(&self) -> &Root {
        &self.sovereign_params.genesis_state_root
    }

    pub fn genesis_da_height(&self) -> Height {
        self.sovereign_params.genesis_da_height
    }

    pub fn code_commitment(&self) -> &CodeCommitment {
        &self.sovereign_params.code_commitment
    }

    pub fn trusting_period(&self) -> Duration {
        self.sovereign_params.trusting_period
    }

    pub fn is_frozen(&self) -> bool {
        self.sovereign_params.frozen_height.is_some()
    }

    pub fn with_frozen_height(self, h: Height) -> Self {
        Self {
            sovereign_params: SovereignClientParams {
                frozen_height: Some(h),
                ..self.sovereign_params
            },
            ..self
        }
    }

    /// Returns latest height of the client state aligned with the rollup's
    /// height (slot number).
    pub fn latest_height_in_sov(&self) -> Height {
        self.sovereign_params.latest_height
    }

    /// Returns the latest height of the client state aligned with the DA
    /// height. This function considers the DA height at which the rollup
    /// started (`genesis_da_height`).
    pub fn latest_height_in_da(&self) -> Height {
        self.latest_height_in_sov()
            .add(self.genesis_da_height().revision_height())
    }

    pub fn upgrade_path(&self) -> &UpgradePath {
        &self.sovereign_params.upgrade_path
    }
}

pub type SovTmClientState = ClientState<TendermintClientParams>;

impl SovTmClientState {
    pub fn chain_id(&self) -> &ChainId {
        &self.da_params.chain_id
    }

    pub fn with_da_header(self, da_header: TmHeader) -> Result<Self, Error> {
        let updating_height = da_header
            .height()
            .sub(self.genesis_da_height().revision_height())
            .map_err(Error::source)?;

        let latest_height = max(updating_height, self.latest_height_in_sov());

        Ok(Self {
            sovereign_params: SovereignClientParams {
                latest_height,
                ..self.sovereign_params
            },
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
            trusting_period: self.sovereign_params.trusting_period,
            clock_drift: self.da_params.max_clock_drift,
        })
    }

    // Resets custom fields to zero values (used in `update_client`)
    pub fn zero_custom_fields(&mut self) {
        self.sovereign_params.frozen_height = None;
        self.sovereign_params.trusting_period = ZERO_DURATION;
        self.da_params.trust_level = TrustThreshold::ZERO;
        self.da_params.max_clock_drift = ZERO_DURATION;
    }
}

impl Protobuf<RawClientState> for SovTmClientState {}

impl TryFrom<RawClientState> for SovTmClientState {
    type Error = ClientError;

    fn try_from(raw: RawClientState) -> Result<Self, Self::Error> {
        let sovereign_params = raw
            .sovereign_params
            .ok_or(Error::missing("sovereign_params"))?
            .try_into()?;

        let tendermint_params = raw
            .tendermint_params
            .ok_or(Error::missing("tendermint_params"))?
            .try_into()?;

        Ok(Self::new(sovereign_params, tendermint_params))
    }
}

impl From<SovTmClientState> for RawClientState {
    fn from(value: SovTmClientState) -> Self {
        Self {
            sovereign_params: Some(value.sovereign_params.into()),
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
