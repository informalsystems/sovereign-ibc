use ibc_core::host::types::identifiers::ChainId;
use ibc_testkit::fixtures::core::signer::dummy_bech32_account;
use sov_modules_api::Address;
use sov_rollup_interface::services::da::DaService;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder, Clone, Debug)]
pub struct TestSetupConfig<Da: DaService> {
    /// The chain Id of the rollup.
    #[builder(default = ChainId::new("mock-rollup-0").unwrap())]
    pub rollup_chain_id: ChainId,
    /// The da service.
    pub da_service: Da,
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
