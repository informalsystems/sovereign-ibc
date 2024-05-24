mod at_counterparty;
mod da_params;
mod definition;

use alloc::str::FromStr;

pub use at_counterparty::ClientStateAtCounterParty;
pub use da_params::*;
pub use definition::*;
use ibc_core::host::types::identifiers::ClientType;

pub const SOV_CELESTIA_CLIENT_TYPE: &str = "100-sov-celestia";

/// Returns the `ClientType` for the `sov-celestia` light client.
pub fn sov_celestia_client_type() -> ClientType {
    ClientType::from_str(SOV_CELESTIA_CLIENT_TYPE).expect("Never fails because it's valid")
}

#[cfg(feature = "test-util")]
pub mod test_util {
    use std::time::Duration;

    use ibc_client_tendermint::types::TrustThreshold;
    use ibc_client_wasm_types::client_state::ClientState as WasmClientState;
    use ibc_client_wasm_types::consensus_state::ConsensusState as WasmConsensusState;
    use ibc_core::client::types::Height;
    use ibc_core::host::types::identifiers::ChainId;
    use ibc_core::primitives::proto::Any;
    use ibc_core::primitives::Timestamp;
    use tendermint::Hash;

    use super::*;
    use crate::consensus_state::{SovTmConsensusState, TmConsensusParams};
    use crate::sovereign::{
        AggregatedProof, SovereignClientParams, SovereignConsensusParams, SovereignParamsConfig,
    };

    pub fn mock_celestia_chain_id() -> ChainId {
        ChainId::new("mock-celestia-0").expect("Never fails")
    }

    #[derive(typed_builder::TypedBuilder, Debug)]
    #[builder(build_method(into = SovTmClientState))]
    pub struct ClientStateConfig {
        pub sovereign_params: SovereignClientParams,
        pub tendermint_params: TendermintClientParams,
    }

    impl From<ClientStateConfig> for SovTmClientState {
        fn from(config: ClientStateConfig) -> Self {
            ClientState::new(config.sovereign_params, config.tendermint_params)
        }
    }

    #[derive(typed_builder::TypedBuilder, Debug)]
    #[builder(build_method(into = TendermintClientParams))]
    pub struct TendermintParamsConfig {
        #[builder(default = mock_celestia_chain_id())]
        pub chain_id: ChainId,
        #[builder(default = TrustThreshold::ONE_THIRD)]
        pub trust_level: TrustThreshold,
        #[builder(default = Duration::from_secs(128000))]
        pub unbonding_period: Duration,
        #[builder(default = Duration::from_millis(3000))]
        pub max_clock_drift: Duration,
    }

    impl From<TendermintParamsConfig> for TendermintClientParams {
        fn from(config: TendermintParamsConfig) -> Self {
            Self::new(
                config.chain_id,
                config.trust_level,
                config.unbonding_period,
                config.max_clock_drift,
            )
        }
    }

    use ibc_client_tendermint::types::Header as TmHeader;

    use crate::client_message::SovTmHeader;
    #[derive(typed_builder::TypedBuilder, Debug)]
    #[builder(build_method(into = SovTmHeader))]
    pub struct HeaderConfig {
        pub da_header: TmHeader,
        pub aggregated_proof: AggregatedProof,
    }

    impl From<HeaderConfig> for SovTmHeader {
        fn from(config: HeaderConfig) -> Self {
            Self {
                da_header: config.da_header,
                aggregated_proof: config.aggregated_proof,
            }
        }
    }

    pub fn dummy_sov_client_state(chain_id: ChainId, latest_height: Height) -> SovTmClientState {
        let sovereign_params = SovereignParamsConfig::builder()
            .latest_height(latest_height)
            .build();

        let tendermint_params = TendermintParamsConfig::builder().chain_id(chain_id).build();

        ClientStateConfig::builder()
            .sovereign_params(sovereign_params)
            .tendermint_params(tendermint_params)
            .build()
    }

    pub fn dummy_sov_consensus_state(timestamp: Timestamp) -> SovTmConsensusState {
        let sovereign_params = SovereignConsensusParams::new(vec![0].into());

        let tendermint_params = TmConsensusParams::new(
            timestamp.into_tm_time().expect("Time exists"),
            // Hash of default validator set
            Hash::from_str("D6B93922C33AAEBEC9043566CB4B1B48365B1358B67C7DEF986D9EE1861BC143")
                .expect("Never fails"),
        );

        SovTmConsensusState::new(sovereign_params, tendermint_params)
    }

    pub fn dummy_wasm_client_state() -> WasmClientState {
        let sovereign_params = SovereignParamsConfig::builder()
            .latest_height(Height::new(0, 1).expect("Never fails"))
            .build();

        let tendermint_params = TendermintParamsConfig::builder()
            .chain_id("test-1".parse().expect("Never fails"))
            .build();

        let client_state = ClientStateConfig::builder()
            .sovereign_params(sovereign_params)
            .tendermint_params(tendermint_params)
            .build();

        WasmClientState {
            data: Any::from(client_state.clone()).value,
            checksum: dummy_checksum(),
            latest_height: client_state.latest_height_in_sov(),
        }
    }

    pub fn dummy_wasm_consensus_state() -> WasmConsensusState {
        WasmConsensusState {
            data: Any::from(dummy_sov_consensus_state(Timestamp::now())).value,
        }
    }

    pub fn dummy_checksum() -> Vec<u8> {
        hex::decode("2469f43c3ca20d476442bd3d98cbd97a180776ab37332aa7b02cae5a620acfc6")
            .expect("Never fails")
    }
}
