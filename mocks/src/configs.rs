use std::fs::File;
use std::io::Read;
use std::path::Path;

use ibc_core::host::types::identifiers::ChainId;
use ibc_testkit::fixtures::core::signer::dummy_bech32_account;
use serde::de::DeserializeOwned;
use sov_bank::{get_token_address, TokenConfig};
#[cfg(feature = "celestia-da")]
use sov_consensus_state_tracker::CelestiaService;
#[cfg(feature = "mock-da")]
use sov_consensus_state_tracker::MockDaService;
use sov_modules_api::{Address, Spec};
use sov_modules_stf_blueprint::kernels::basic::BasicKernelGenesisConfig;
use sov_rollup_interface::services::da::DaService;
use typed_builder::TypedBuilder;

pub(crate) type DefaultSpec =
    sov_modules_api::default_spec::DefaultSpec<sov_mock_zkvm::MockZkVerifier>;

#[cfg(feature = "celestia-da")]
use crate::sovereign::celestia_da_service;
#[cfg(feature = "mock-da")]
use crate::sovereign::mock_da_service;
use crate::sovereign::{GenesisConfig, RollupGenesisConfig};

#[derive(TypedBuilder, Clone, Debug)]
pub struct TestSetupConfig<S: Spec, Da: DaService> {
    /// The chain Id of the DA chain.
    #[builder(default = ChainId::new("mock-celestia-0").unwrap())]
    pub da_chain_id: ChainId,
    /// The da service.
    pub da_service: Da,
    /// The chain Id of the rollup.
    #[builder(default = ChainId::new("mock-rollup-0").unwrap())]
    pub rollup_id: ChainId,
    /// The runtime configuration.
    #[builder(default = RollupGenesisConfig::default())]
    pub rollup_genesis_config: RollupGenesisConfig<S>,
    /// Sets whether to use manual IBC TAO or not.
    #[builder(default = false)]
    pub with_manual_tao: bool,
}

impl<S: Spec, Da: DaService> TestSetupConfig<S, Da> {
    /// Returns list of tokens in the bank configuration
    pub fn get_tokens(&self) -> &Vec<TokenConfig<S>> {
        &self.rollup_genesis_config.bank_config.tokens
    }

    /// Returns the address of the relayer. We use the last address in the list
    /// as the relayer address
    pub fn get_relayer_address(&self) -> S::Address {
        self.rollup_genesis_config.bank_config.tokens[0]
            .address_and_balances
            .last()
            .unwrap()
            .0
            .clone()
    }

    /// Returns the token address for a given token configuration
    pub fn get_token_address(&self, token_cfg: &TokenConfig<S>) -> S::Address {
        get_token_address::<S>(
            &token_cfg.token_name,
            &self.get_relayer_address(),
            token_cfg.salt,
        )
    }

    pub fn kernel_genesis_config(&self) -> BasicKernelGenesisConfig<S, Da::Spec> {
        BasicKernelGenesisConfig {
            chain_state: self.rollup_genesis_config.chain_state_config.clone(),
        }
    }

    pub fn runtime_genesis_config(&self) -> GenesisConfig<S, Da::Spec> {
        GenesisConfig::new(
            self.rollup_genesis_config.bank_config.clone(),
            self.rollup_genesis_config.ibc_config.clone(),
            self.rollup_genesis_config.ibc_transfer_config.clone(),
        )
    }
}

#[cfg(feature = "mock-da")]
pub fn default_config_with_mock_da() -> TestSetupConfig<DefaultSpec, MockDaService> {
    TestSetupConfig::<DefaultSpec, MockDaService>::builder()
        .da_service(mock_da_service())
        .build()
}

#[cfg(feature = "celestia-da")]
pub async fn default_config_with_celestia_da() -> TestSetupConfig<DefaultSpec, CelestiaService> {
    TestSetupConfig::<DefaultSpec, CelestiaService>::builder()
        .da_service(celestia_da_service().await)
        .build()
}

/// Configuration for the `transfer` tests.
#[derive(TypedBuilder, Clone, Debug)]
pub struct TransferTestConfig {
    /// The token name on the rollup.
    pub sov_denom: String,
    /// The token address on the rollup.
    #[builder(default = None)]
    pub sov_token_address: Option<Address>,
    /// An arbitrary user address on the rollup.
    pub sov_address: Address,
    /// The token name on the Cosmos chain.
    #[builder(default = "basecoin".to_string())]
    pub cos_denom: String,
    /// An arbitrary user address on the Cosmos chain.
    #[builder(default = dummy_bech32_account())]
    pub cos_address: String,
    /// The amount to transfer.
    #[builder(default = 100)]
    pub amount: u64,
}

/// Reads toml file as a specific type.
pub fn from_toml_path<P: AsRef<Path>, R: DeserializeOwned>(path: P) -> anyhow::Result<R> {
    let mut contents = String::new();

    {
        let mut file = File::open(path)?;
        file.read_to_string(&mut contents)?;
    }

    let result: R = toml::from_str(&contents)?;

    Ok(result)
}
