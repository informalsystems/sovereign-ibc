use core::cmp::max;

use ibc_client_tendermint::types::{Header as TmHeader, TrustThreshold};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_core::primitives::ZERO_DURATION;
use sov_ibc_proto::ibc::lightclients::sovereign::tendermint::v1::ClientState as RawClientState;

use super::TmClientParams;
use crate::error::Error;

pub const SOV_TENDERMINT_CLIENT_STATE_TYPE_URL: &str =
    "/ibc.lightclients.sovereign.tendermint.v1.ClientState";

/// Contains the core implementation of the Sovereign light client
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClientState<Da> {
    pub rollup_id: ChainId,
    pub latest_height: Height,
    pub frozen_height: Option<Height>,
    pub upgrade_path: Vec<String>,
    pub da_params: Da,
}

impl<Da> ClientState<Da> {
    pub fn new(
        rollup_id: ChainId,
        latest_height: Height,
        upgrade_path: Vec<String>,
        da_params: Da,
    ) -> Self {
        Self {
            rollup_id,
            latest_height,
            frozen_height: None,
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

        if raw.frozen_height.is_some() {
            return Err(Error::invalid("frozen_height is not supported"))?;
        }

        let upgrade_path = raw.upgrade_path;

        let tendermint_params = raw
            .tendermint_params
            .ok_or(Error::missing("tendermint_params"))?
            .try_into()?;

        Ok(Self::new(
            rollup_id,
            latest_height,
            upgrade_path,
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
            upgrade_path: value.upgrade_path,
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
