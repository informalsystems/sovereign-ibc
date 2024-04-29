use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use sov_bank::{BankConfig, GasTokenConfig};
use sov_chain_state::ChainStateConfig;
use sov_ibc::ExampleModuleConfig;
use sov_ibc_transfer::TransferConfig;
use sov_modules_api::utils::generate_address as gen_address_generic;
use sov_modules_api::{Gas, GasArray, Spec, Zkvm};
use sov_rollup_interface::da::Time;
use sov_rollup_interface::zk::CodeCommitment;

/// The default initial slot height.
pub const DEFAULT_INIT_HEIGHT: u64 = 1;

/// The default initial balance for each address.
pub const DEFAULT_INIT_BALANCE: u64 = 1000;

/// The default number of addresses.
pub const DEFAULT_ADDRESS_COUNT: u64 = 3;

/// The default gas token name.
pub const DEFAULT_GAS_TOKEN_NAME: &str = "sov-gas-token";

/// The default salt.
pub const DEFAULT_SALT: u64 = 0;

pub struct RollupGenesisConfig<S: Spec> {
    pub chain_state_config: ChainStateConfig<S>,
    pub bank_config: BankConfig<S>,
    pub ibc_config: ExampleModuleConfig,
    pub ibc_transfer_config: TransferConfig,
}

impl<S: Spec> RollupGenesisConfig<S> {
    pub fn cloned_chain_state_config(&self) -> ChainStateConfig<S> {
        ChainStateConfig {
            current_time: self.chain_state_config.current_time.clone(),
            initial_base_fee_per_gas: self.chain_state_config.initial_base_fee_per_gas.clone(),
            genesis_da_height: self.chain_state_config.genesis_da_height.clone(),
            inner_code_commitment: self.chain_state_config.inner_code_commitment.clone(),
            outer_code_commitment: self.chain_state_config.outer_code_commitment.clone(),
        }
    }
}

impl<S: Spec> Clone for RollupGenesisConfig<S> {
    fn clone(&self) -> Self {
        Self {
            chain_state_config: self.cloned_chain_state_config(),
            bank_config: self.bank_config.clone(),
            ibc_config: self.ibc_config.clone(),
            ibc_transfer_config: self.ibc_transfer_config.clone(),
        }
    }
}

impl<S: Spec> fmt::Debug for RollupGenesisConfig<S>
where
    <S::InnerZkvm as Zkvm>::CodeCommitment: fmt::Debug,
    <S::OuterZkvm as Zkvm>::CodeCommitment: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RollupGenesisConfig")
            .field(
                "chain_state_config.current_time",
                &self.chain_state_config.current_time,
            )
            .field(
                "chain_state_config.initial_base_fee_per_gas",
                &self.chain_state_config.initial_base_fee_per_gas,
            )
            .field(
                "chain_state_config.inner_code_commitment",
                &self.chain_state_config.inner_code_commitment,
            )
            .field(
                "chain_state_config.outer_code_commitment",
                &self.chain_state_config.outer_code_commitment,
            )
            .field(
                "chain_state_config.genesis_da_height",
                &self.chain_state_config.genesis_da_height,
            )
            .field("bank_config", &self.bank_config)
            .field("ibc_config", &self.ibc_config)
            .field("ibc_transfer_config", &self.ibc_transfer_config)
            .finish()
    }
}

impl<S: Spec> RollupGenesisConfig<S> {
    pub fn new(
        chain_state_config: ChainStateConfig<S>,
        bank_config: BankConfig<S>,
        ibc_config: ExampleModuleConfig,
        ibc_transfer_config: TransferConfig,
    ) -> Self {
        Self {
            chain_state_config,
            bank_config,
            ibc_config,
            ibc_transfer_config,
        }
    }
}

impl<S: Spec> Default for RollupGenesisConfig<S> {
    fn default() -> Self {
        let chain_state_config = create_chain_state_config();

        let bank_config = create_bank_config(DEFAULT_ADDRESS_COUNT, DEFAULT_INIT_BALANCE);

        let ibc_config = ExampleModuleConfig {};

        let ibc_transfer_config = TransferConfig {};

        Self {
            chain_state_config,
            bank_config,
            ibc_config,
            ibc_transfer_config,
        }
    }
}

/// Creates a chain state configuration with the given initial slot height
pub fn create_chain_state_config<S: Spec>() -> ChainStateConfig<S> {
    let now = SystemTime::now();

    let seconds = now
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    ChainStateConfig {
        current_time: Time::from_secs(seconds.try_into().unwrap()),
        initial_base_fee_per_gas: <S::Gas as Gas>::Price::ZEROED,
        genesis_da_height: 0,
        inner_code_commitment: <S::InnerZkvm as Zkvm>::CodeCommitment::decode(&[]).unwrap(),
        outer_code_commitment: <S::OuterZkvm as Zkvm>::CodeCommitment::decode(&[]).unwrap(),
    }
}

/// Creates a bank configuration with the given number of addresses and initial balance
pub fn create_bank_config<S: Spec>(addresses_count: u64, initial_balance: u64) -> BankConfig<S> {
    let address_and_balances: Vec<_> = (0..addresses_count)
        .map(|i| {
            let key = format!("key_{i}");
            let addr = gen_address_generic::<S>(&key);
            (addr, initial_balance)
        })
        .collect();

    let gas_token_config = GasTokenConfig {
        token_name: DEFAULT_GAS_TOKEN_NAME.to_owned(),
        address_and_balances: address_and_balances.clone(),
        authorized_minters: vec![],
    };

    BankConfig {
        gas_token_config,
        tokens: vec![],
    }
}
