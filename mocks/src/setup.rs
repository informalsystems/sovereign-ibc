use std::time::Duration;

use basecoin_store::impls::InMemoryStore;
use ibc_client_tendermint::types::client_type as tm_client_type;
use ibc_core::host::types::identifiers::{ClientId, Sequence};
use ibc_core::host::ValidationContext;
use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::{Context, ModuleInfo, WorkingSet};
use sov_rollup_interface::services::da::DaService;
use sov_state::{DefaultStorageSpec, ProverStorage};
use tokio::time::sleep;
use tracing::info;

use super::cosmos::dummy_signer;
use super::relayer::DefaultRelayer;
use crate::configs::TestSetupConfig;
use crate::cosmos::CosmosBuilder;
use crate::relayer::handle::{Handle, QueryReq, QueryResp};
use crate::relayer::relay::MockRelayer;
use crate::relayer::DefaultRollup;
use crate::sovereign::{MockRollup, Runtime, RuntimeConfig, DEFAULT_INIT_HEIGHT};

/// Initializes a mock rollup and a mock Cosmos chain and sets up the relayer between them.
pub async fn setup<Da>(
    setup_cfg: TestSetupConfig<Da>,
    with_manual_tao: bool,
) -> (DefaultRelayer<Da>, DefaultRollup<Da>)
where
    Da: DaService<Error = anyhow::Error> + Clone,
    MockRollup<DefaultContext, Da, DefaultStorageSpec>: Handle,
{
    let config = RuntimeConfig::default();

    let runtime = Runtime::default();

    // Set the default sender address to the address of the 'sov-ibc-transfer'
    // module, ensuring that the module's address is used for the token creation.
    let rollup_ctx = DefaultContext::new(*runtime.ibc_transfer.address(), DEFAULT_INIT_HEIGHT);

    let path = tempfile::tempdir().unwrap();

    let prover_storage = ProverStorage::with_path(path).unwrap();

    let mut rollup = MockRollup::new(
        setup_cfg.rollup_chain_id,
        config,
        runtime,
        prover_storage,
        rollup_ctx,
        setup_cfg.da_service,
    );

    rollup.init_chain().await;

    let sov_client_counter = match rollup.query(QueryReq::ClientCounter) {
        QueryResp::ClientCounter(counter) => counter,
        _ => panic!("Unexpected response"),
    };

    // TODO: this should be updated when there is a light client for sovereign chains
    let sov_client_id = ClientId::new(tm_client_type(), sov_client_counter).unwrap();

    let mut cos_builder = CosmosBuilder::default();

    let mut cos_chain = cos_builder.build_chain(InMemoryStore::default());

    wait_for_block().await;

    info!("cosmos: initialized with chain id {}", cos_chain.chain_id());

    let cos_client_counter = cos_chain.ibc_ctx().client_counter().unwrap();

    let cos_client_id = ClientId::new(tm_client_type(), cos_client_counter).unwrap();

    if with_manual_tao {
        let sov_client_id = rollup.setup_client(cos_chain.chain_id()).await;
        let cos_client_id = cos_chain.setup_client(rollup.chain_id());

        let sov_conn_id = rollup
            .setup_connection(sov_client_id, cos_chain.ibc_ctx().commitment_prefix())
            .await;

        let mut working_set = WorkingSet::new(rollup.prover_storage());
        let cos_conn_id = cos_chain.setup_connection(
            cos_client_id,
            rollup.ibc_ctx(&mut working_set).commitment_prefix(),
        );

        let (sov_port_id, sov_chan_id) = rollup.setup_channel(sov_conn_id).await;
        let (cos_port_id, cos_chan_id) = cos_chain.setup_channel(cos_conn_id);

        rollup
            .with_send_sequence(sov_port_id, sov_chan_id, Sequence::from(1))
            .await;
        cos_chain.with_send_sequence(cos_port_id, cos_chan_id, Sequence::from(1));

        info!("relayer: manually initialized IBC TAO layers");
    }

    rollup.run().await;

    wait_for_block().await;

    info!("rollup: initialized with chain id {}", rollup.chain_id());

    (
        MockRelayer::new(
            rollup.clone().into(),
            cos_chain.into(),
            sov_client_id,
            cos_client_id,
            rollup.get_relayer_address().to_string().into(),
            dummy_signer(),
        ),
        rollup,
    )
}

/// Waits for the mock chains to generate a few blocks.
pub async fn wait_for_block() {
    sleep(Duration::from_secs(1)).await;
}
