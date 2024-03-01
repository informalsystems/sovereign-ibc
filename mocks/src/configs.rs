use std::fs::File;
use std::io::Read;
use std::path::Path;

use ibc_core::host::types::identifiers::ChainId;
use ibc_testkit::fixtures::core::signer::dummy_bech32_account;
use serde::de::DeserializeOwned;
use sov_bank::{get_token_address, TokenConfig};
use sov_celestia_adapter::CelestiaService;
use sov_consensus_state_tracker::HasConsensusState;
use sov_mock_da::MockDaService;
use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::{Address, Context};
use sov_modules_stf_blueprint::kernels::basic::BasicKernelGenesisConfig;
use sov_rollup_interface::services::da::DaService;
use typed_builder::TypedBuilder;

use crate::sovereign::{celestia_da_service, mock_da_service, GenesisConfig, RollupGenesisConfig};

#[derive(TypedBuilder, Clone, Debug)]
pub struct TestSetupConfig<C: Context, Da: DaService> {
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
    pub rollup_genesis_config: RollupGenesisConfig<C>,
    /// Sets whether to use manual IBC TAO or not.
    #[builder(default = false)]
    pub with_manual_tao: bool,
}

impl<C: Context, Da: DaService> TestSetupConfig<C, Da>
where
    Da::Spec: HasConsensusState,
{
    /// Returns list of tokens in the bank configuration
    pub fn get_tokens(&self) -> &Vec<TokenConfig<C>> {
        &self.rollup_genesis_config.bank_config.tokens
    }

    /// Returns the address of the relayer. We use the last address in the list
    /// as the relayer address
    pub fn get_relayer_address(&self) -> C::Address {
        self.rollup_genesis_config.bank_config.tokens[0]
            .address_and_balances
            .last()
            .unwrap()
            .0
            .clone()
    }

    /// Returns the token address for a given token configuration
    pub fn get_token_address(&self, token_cfg: &TokenConfig<C>) -> C::Address {
        get_token_address::<C>(
            &token_cfg.token_name,
            self.get_relayer_address().as_ref(),
            token_cfg.salt,
        )
    }

    pub fn kernel_genesis_config(&self) -> BasicKernelGenesisConfig<C, Da::Spec> {
        BasicKernelGenesisConfig {
            chain_state: self.rollup_genesis_config.chain_state_config.clone(),
        }
    }

    pub fn runtime_genesis_config(&self) -> GenesisConfig<C, Da::Spec> {
        GenesisConfig::new(
            self.rollup_genesis_config.bank_config.clone(),
            self.rollup_genesis_config.ibc_config.clone(),
            self.rollup_genesis_config.ibc_transfer_config.clone(),
        )
    }
}

pub fn default_config_with_mock_da() -> TestSetupConfig<DefaultContext, MockDaService> {
    TestSetupConfig::<DefaultContext, MockDaService>::builder()
        .da_service(mock_da_service())
        .build()
}

pub async fn default_config_with_celestia_da() -> TestSetupConfig<DefaultContext, CelestiaService> {
    TestSetupConfig::<DefaultContext, CelestiaService>::builder()
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
