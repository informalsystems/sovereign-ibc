use core::cmp::max;

use ibc_client_tendermint::types::{Header as TmHeader, TrustThreshold};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_core::primitives::ZERO_DURATION;
use ibc_proto::ibc::lightclients::sovereign::tendermint::v1::ClientState as RawClientState;

use super::TendermintParams;
use crate::error::Error;

pub const SOV_TENDERMINT_CLIENT_STATE_TYPE_URL: &str =
    "/ibc.lightclients.sovereign.tendermint.v1.ClientState";

/// Contains the core implementation of the Sovereign light client
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClientState {
    pub rollup_id: ChainId,
    pub latest_height: Height,
    pub frozen_height: Option<Height>,
    pub upgrade_path: Vec<String>,
    pub tendermint_params: TendermintParams,
}

impl ClientState {
    pub fn new(
        rollup_id: ChainId,
        latest_height: Height,
        upgrade_path: Vec<String>,
        tendermint_params: TendermintParams,
    ) -> Self {
        Self {
            rollup_id,
            latest_height,
            frozen_height: None,
            upgrade_path,
            tendermint_params,
        }
    }

    pub fn rollup_id(&self) -> &ChainId {
        &self.rollup_id
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.tendermint_params.chain_id
    }

    pub fn latest_height(&self) -> Height {
        self.latest_height
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen_height.is_some()
    }

    pub fn with_header(self, header: TmHeader) -> Result<Self, Error> {
        Ok(Self {
            latest_height: max(header.height(), self.latest_height),
            ..self
        })
    }

    pub fn with_frozen_height(self, h: Height) -> Self {
        Self {
            frozen_height: Some(h),
            ..self
        }
    }

    // Resets custom fields to zero values (used in `update_client`)
    pub fn zero_custom_fields(&mut self) {
        self.frozen_height = None;
        self.tendermint_params.trusting_period = ZERO_DURATION;
        self.tendermint_params.trust_level = TrustThreshold::ZERO;
        self.tendermint_params.max_clock_drift = ZERO_DURATION;
    }
}

impl Protobuf<RawClientState> for ClientState {}

impl TryFrom<RawClientState> for ClientState {
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

impl From<ClientState> for RawClientState {
    fn from(value: ClientState) -> Self {
        Self {
            rollup_id: value.rollup_id.to_string(),
            latest_height: Some(value.latest_height.into()),
            frozen_height: value.frozen_height.map(|h| h.into()),
            upgrade_path: value.upgrade_path,
            tendermint_params: Some(value.tendermint_params.into()),
        }
    }
}

impl Protobuf<Any> for ClientState {}

impl TryFrom<Any> for ClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        fn decode_client_state(value: &[u8]) -> Result<ClientState, ClientError> {
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

impl From<ClientState> for Any {
    fn from(client_state: ClientState) -> Self {
        Any {
            type_url: SOV_TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawClientState>::encode_vec(client_state),
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_util {
    use core::time::Duration;

    use ibc_client_tendermint::types::TrustThreshold;
    use ibc_client_wasm_types::client_state::ClientState as WasmClientState;
    use ibc_core::client::types::Height;
    use ibc_core::host::types::identifiers::ChainId;

    use super::*;
    use crate::Bytes;

    #[derive(typed_builder::TypedBuilder, Debug)]
    pub struct ClientStateConfig {
        pub rollup_id: ChainId,
        pub latest_height: Height,
        #[builder(default)]
        pub frozen_height: Option<Height>,
        #[builder(default)]
        pub upgrade_path: Vec<String>,
        pub tendermint_params: TendermintParams,
    }

    #[derive(typed_builder::TypedBuilder, Debug)]
    pub struct TendermintParamsConfig {
        pub chain_id: ChainId,
        #[builder(default)]
        pub trust_level: TrustThreshold,
        #[builder(default = Duration::from_secs(64000))]
        pub trusting_period: Duration,
        #[builder(default = Duration::from_secs(128000))]
        pub unbonding_period: Duration,
        #[builder(default = Duration::from_millis(3000))]
        max_clock_drift: Duration,
        pub latest_height: Height,
        #[builder(default)]
        pub upgrade_path: Vec<String>,
    }

    impl TryFrom<TendermintParamsConfig> for TendermintParams {
        type Error = Error;

        fn try_from(config: TendermintParamsConfig) -> Result<Self, Self::Error> {
            Ok(Self::new(
                config.chain_id,
                config.trust_level,
                config.trusting_period,
                config.unbonding_period,
                config.max_clock_drift,
            ))
        }
    }

    impl TryFrom<ClientStateConfig> for ClientState {
        type Error = Error;

        fn try_from(config: ClientStateConfig) -> Result<Self, Self::Error> {
            Ok(Self::new(
                config.rollup_id,
                config.latest_height,
                config.upgrade_path,
                config.tendermint_params,
            ))
        }
    }

    impl ClientState {
        pub fn into_wasm(&self, checksum: Bytes) -> WasmClientState {
            WasmClientState {
                data: Any::from(self.clone()).value,
                checksum,
                latest_height: self.latest_height,
            }
        }
    }

    pub fn create_wasm_msg() -> WasmClientState {
        let tendermint_params: TendermintParams = TendermintParamsConfig::builder()
            .chain_id("testnet".parse().unwrap())
            .latest_height(Height::new(0, 1).unwrap())
            .build()
            .try_into()
            .unwrap();

        let code_hash =
            hex::decode("602901fd7018260b063488dd07261fe8f926670223197beb873f91b51ee09f8e")
                .unwrap();

        let client_state: ClientState = ClientStateConfig::builder()
            .rollup_id("rollup".parse().unwrap())
            .latest_height(Height::new(0, 1).unwrap())
            .tendermint_params(tendermint_params)
            .build()
            .try_into()
            .unwrap();

        client_state.into_wasm(code_hash)
    }
}

#[cfg(test)]
mod tests {

    use ibc_client_wasm_types::client_state::ClientState;
    use ibc_core::primitives::ToProto;
    use ibc_proto::ibc::lightclients::wasm::v1::ClientState as RawClientState;

    use crate::client_state::definition::test_util;

    #[test]
    fn create_wasm_msg() {
        let wasm_msg: ClientState = test_util::create_wasm_msg();

        let any_wasm_msg = <ClientState as ToProto<RawClientState>>::to_any(wasm_msg);

        println!("wasm_msg: {:?}", any_wasm_msg);
    }
}
