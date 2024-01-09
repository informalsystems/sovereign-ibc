use ibc_client_tendermint::types::ClientState as TmClientState;
use ibc_core::client::types::error::ClientError;
use ibc_core::primitives::proto::Any;
use ibc_proto::ibc::lightclients::sovereign::tendermint::v1::SovTmClientState as RawSovTmClientState;
use tendermint_proto::Protobuf;

use super::RollupClientState;
use crate::error::Error;

pub const SOV_TENDERMINT_CLIENT_STATE_TYPE_URL: &str =
    "/ibc.lightclients.sovereign.tendermint.v1.ClientState";

/// Contains the core implementation of the Sovereign light client
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SovTmClientState {
    pub da_client_state: TmClientState,
    pub rollup_client_state: RollupClientState,
}

impl SovTmClientState {
    pub fn new(da_client_state: TmClientState, rollup_client_state: RollupClientState) -> Self {
        Self {
            da_client_state,
            rollup_client_state,
        }
    }
}

impl Protobuf<RawSovTmClientState> for SovTmClientState {}

impl TryFrom<RawSovTmClientState> for SovTmClientState {
    type Error = ClientError;

    fn try_from(raw: RawSovTmClientState) -> Result<Self, Self::Error> {
        let da_client_state = TmClientState::try_from(
            raw.da_client_state
                .ok_or(Error::invalid("missing sovereign client state"))?,
        )?;

        let rollup_client_state = RollupClientState::try_from(
            raw.rollup_client_state
                .ok_or(Error::invalid("missing rollup client state"))?,
        )?;

        Ok(SovTmClientState {
            da_client_state,
            rollup_client_state,
        })
    }
}

impl From<SovTmClientState> for RawSovTmClientState {
    fn from(value: SovTmClientState) -> Self {
        RawSovTmClientState {
            da_client_state: Some(value.da_client_state.into()),
            rollup_client_state: Some(value.rollup_client_state.into()),
        }
    }
}

impl Protobuf<Any> for SovTmClientState {}

impl TryFrom<Any> for SovTmClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        fn decode_client_state(value: &[u8]) -> Result<SovTmClientState, ClientError> {
            let client_state =
                Protobuf::<RawSovTmClientState>::decode(value).map_err(|e| ClientError::Other {
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
            value: Protobuf::<RawSovTmClientState>::encode_vec(client_state),
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_util {
    use core::time::Duration;

    use ibc_client_tendermint::types::{AllowUpdate, TrustThreshold};
    use ibc_client_wasm_types::client_state::ClientState as WasmClientState;
    use ibc_core::client::types::Height;
    use ibc_core::commitment_types::specs::ProofSpecs;
    use ibc_core::host::types::identifiers::ChainId;

    use super::*;
    use crate::Bytes;

    #[derive(typed_builder::TypedBuilder, Debug)]
    pub struct TmClientStateConfig {
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
        pub proof_specs: ProofSpecs,
        #[builder(default)]
        pub upgrade_path: Vec<String>,
        #[builder(default = AllowUpdate { after_expiry: false, after_misbehaviour: false })]
        allow_update: AllowUpdate,
    }

    #[derive(typed_builder::TypedBuilder, Debug)]
    pub struct RollupClientStateConfig {
        pub rollup_id: ChainId,
        #[builder(default)]
        pub post_root_state: Vec<u8>,
    }

    impl TryFrom<TmClientStateConfig> for TmClientState {
        type Error = Error;

        fn try_from(config: TmClientStateConfig) -> Result<Self, Self::Error> {
            TmClientState::new(
                config.chain_id,
                config.trust_level,
                config.trusting_period,
                config.unbonding_period,
                config.max_clock_drift,
                config.latest_height,
                config.proof_specs,
                config.upgrade_path,
                config.allow_update,
            )
            .map_err(Error::source)
        }
    }

    impl TryFrom<RollupClientStateConfig> for RollupClientState {
        type Error = Error;

        fn try_from(config: RollupClientStateConfig) -> Result<Self, Self::Error> {
            Ok(RollupClientState::new(
                config.rollup_id,
                config.post_root_state,
            ))
        }
    }

    impl SovTmClientState {
        pub fn into_wasm(&self, checksum: Bytes) -> WasmClientState {
            WasmClientState {
                data: Any::from(self.clone()).value,
                checksum,
                latest_height: self.da_client_state.latest_height,
            }
        }
    }

    pub fn create_wasm_msg() -> WasmClientState {
        let tm_client_state: TmClientState = TmClientStateConfig::builder()
            .chain_id("testnet".parse().unwrap())
            .latest_height(Height::new(0, 1).unwrap())
            .build()
            .try_into()
            .unwrap();

        let code_hash =
            hex::decode("602901fd7018260b063488dd07261fe8f926670223197beb873f91b51ee09f8e")
                .unwrap();

        let rollup_client_state = RollupClientStateConfig::builder()
            .rollup_id("rollup".parse().unwrap())
            .post_root_state(vec![0])
            .build()
            .try_into()
            .unwrap();

        let sov_tm_client_state = SovTmClientState {
            da_client_state: tm_client_state,
            rollup_client_state,
        };

        sov_tm_client_state.into_wasm(code_hash)
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
