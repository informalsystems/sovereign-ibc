use std::time::Duration;

use ibc_client_tendermint::types::TrustThreshold;
use ibc_core::client::types::Height;
use ibc_core::host::types::identifiers::ChainId;
use sov_celestia_client::client_state::ClientState;
use sov_celestia_client::types::client_state::{ClientState as ClientStateType, TendermintParams};

#[derive(typed_builder::TypedBuilder, Debug)]
#[builder(build_method(into = ClientState))]
pub struct ClientStateConfig {
    pub rollup_id: ChainId,
    pub latest_height: Height,
    #[builder(default)]
    pub frozen_height: Option<Height>,
    #[builder(default)]
    pub upgrade_path: Vec<String>,
    pub tendermint_params: TendermintParams,
}

impl From<ClientStateConfig> for ClientState {
    fn from(config: ClientStateConfig) -> Self {
        ClientStateType::new(
            config.rollup_id,
            config.latest_height,
            config.upgrade_path,
            config.tendermint_params,
        )
        .into()
    }
}

#[derive(typed_builder::TypedBuilder, Debug)]
#[builder(build_method(into = TendermintParams))]
pub struct TendermintParamsConfig {
    #[builder(default = ChainId::new("ibc-0").expect("Never fails"))]
    pub chain_id: ChainId,
    #[builder(default)]
    pub trust_level: TrustThreshold,
    #[builder(default = Duration::from_secs(64000))]
    pub trusting_period: Duration,
    #[builder(default = Duration::from_secs(128000))]
    pub unbonding_period: Duration,
    #[builder(default = Duration::from_millis(3000))]
    pub max_clock_drift: Duration,
}

impl From<TendermintParamsConfig> for TendermintParams {
    fn from(config: TendermintParamsConfig) -> Self {
        Self::new(
            config.chain_id,
            config.trust_level,
            config.trusting_period,
            config.unbonding_period,
            config.max_clock_drift,
        )
    }
}

pub fn dummy_sov_client_state(rollup_id: ChainId, current_height: Height) -> ClientState {
    let tendermint_params = TendermintParamsConfig::builder().build();

    ClientStateConfig::builder()
        .rollup_id(rollup_id)
        .latest_height(current_height)
        .tendermint_params(tendermint_params)
        .build()
}
