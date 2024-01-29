use std::time::Duration;

use ibc_client_tendermint::types::{Header as TmHeader, TrustThreshold};
use ibc_core::client::types::Height;
use ibc_core::host::types::identifiers::ChainId;
use sov_celestia_client::client_state::ClientState;
use sov_celestia_client::types::client_message::{
    AggregatedProof, AggregatedProofData, ProofDataInfo, PublicInput, SovTmHeader,
};
use sov_celestia_client::types::client_state::{ClientState as ClientStateType, TendermintParams};
use sov_celestia_client::types::proto::types::v1::AggregatedProof as RawAggregatedProof;
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
    #[builder(default = ChainId::new("mock-celestia-0").expect("Never fails"))]
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

pub fn dummy_sov_client_state(rollup_id: ChainId, latest_height: Height) -> ClientState {
    let tendermint_params = TendermintParamsConfig::builder().build();

    ClientStateConfig::builder()
        .rollup_id(rollup_id)
        .latest_height(latest_height)
        .tendermint_params(tendermint_params)
        .build()
}

#[derive(typed_builder::TypedBuilder, Debug)]
#[builder(build_method(into = SovTmHeader))]
pub struct HeaderConfig {
    pub da_header: TmHeader,
    pub aggregated_proof_data: AggregatedProofData,
}

impl From<HeaderConfig> for SovTmHeader {
    fn from(config: HeaderConfig) -> Self {
        Self {
            da_header: config.da_header,
            aggregated_proof_data: config.aggregated_proof_data,
        }
    }
}

#[derive(typed_builder::TypedBuilder, Debug)]
#[builder(build_method(into = AggregatedProofData))]
pub struct AggregatedProofDataConfig {
    #[builder(default)]
    pub public_input: PublicInputConfig,
    pub proof_data_info: ProofDataInfoConfig,
    #[builder(default)]
    pub aggregated_proof: AggregatedProofConfig,
}

impl From<AggregatedProofDataConfig> for AggregatedProofData {
    fn from(config: AggregatedProofDataConfig) -> Self {
        Self {
            public_input: config.public_input.into(),
            proof_data_info: config.proof_data_info.into(),
            aggregated_proof: config.aggregated_proof.into(),
        }
    }
}

#[derive(typed_builder::TypedBuilder, Debug, Default)]
pub struct PublicInputConfig {
    #[builder(default)]
    pub initial_da_block_hash: Vec<u8>,
    #[builder(default)]
    pub final_da_block_hash: Vec<u8>,
    #[builder(default)]
    pub input_state_root: Vec<u8>,
    #[builder(default)]
    pub final_state_root: Vec<u8>,
}

impl From<PublicInputConfig> for PublicInput {
    fn from(config: PublicInputConfig) -> Self {
        Self {
            initial_da_block_hash: config.initial_da_block_hash,
            final_da_block_hash: config.final_da_block_hash,
            input_state_root: config.input_state_root,
            final_state_root: config.final_state_root,
        }
    }
}

#[derive(typed_builder::TypedBuilder, Debug)]
pub struct ProofDataInfoConfig {
    pub initial_state_height: Height,
    pub final_state_height: Height,
}

impl From<ProofDataInfoConfig> for ProofDataInfo {
    fn from(config: ProofDataInfoConfig) -> Self {
        Self {
            initial_state_height: config.initial_state_height,
            final_state_height: config.final_state_height,
        }
    }
}

#[derive(typed_builder::TypedBuilder, Debug, Default)]
pub struct AggregatedProofConfig {
    #[builder(default)]
    pub proof: Vec<u8>,
}

impl From<AggregatedProofConfig> for AggregatedProof {
    fn from(config: AggregatedProofConfig) -> Self {
        Self::from(RawAggregatedProof {
            proof: config.proof,
        })
    }
}

pub fn dummy_sov_header(
    da_header: TmHeader,
    initial_state_height: Height,
    final_state_height: Height,
) -> SovTmHeader {
    let aggregated_proof_data = AggregatedProofDataConfig::builder()
        .proof_data_info(
            ProofDataInfoConfig::builder()
                .initial_state_height(initial_state_height)
                .final_state_height(final_state_height)
                .build(),
        )
        .build();

    HeaderConfig::builder()
        .da_header(da_header)
        .aggregated_proof_data(aggregated_proof_data)
        .build()
}
