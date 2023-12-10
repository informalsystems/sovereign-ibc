use std::time::Duration;

use basecoin_store::impls::InMemoryStore;
use ibc_client_tendermint::types::client_type as tm_client_type;
use ibc_core::host::types::identifiers::{ChainId, ClientId, Sequence};
use ibc_core::host::ValidationContext;
use sov_mock_da::{MockAddress, MockDaService};
use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::{Context, ModuleInfo, WorkingSet};
use sov_state::{DefaultStorageSpec, ProverStorage};
use tokio::time::sleep;
use tracing::info;

use super::cosmos::helpers::dummy_signer;
use super::relayer::Relayer;
use crate::cosmos::builder::CosmosBuilder;
use crate::relayer::handle::{Handle, QueryReq, QueryResp};
use crate::relayer::relay::MockRelayer;
use crate::sovereign::config::TestConfig;
use crate::sovereign::rollup::MockRollup;
use crate::sovereign::runtime::Runtime;

/// Set ups a relayer between a mock rollup and a mock cosmos chain
pub async fn setup<'ws>(
    with_manual_tao: bool,
) -> (
    Relayer<'ws>,
    MockRollup<DefaultContext, MockDaService, DefaultStorageSpec>,
) {
    let rollup_chain_id = ChainId::new("mock-rollup-0").unwrap();

    let config = TestConfig::default();

    let runtime = Runtime::default();

    // Set the default sender address to the address of the 'sov-ibc-transfer'
    // module, ensuring that the module's address is used for token creation.
    let rollup_ctx = DefaultContext::new(*runtime.ibc_transfer.address(), 0);

    let da_service = MockDaService::new(MockAddress::default());

    let path = tempfile::tempdir().unwrap();

    let prover_storage = ProverStorage::with_path(path).unwrap();

    let mut rollup = MockRollup::new(
        rollup_chain_id,
        config,
        runtime,
        prover_storage,
        rollup_ctx,
        da_service,
    );

    rollup.init_chain().await;

    info!("rollup: initialized with chain id {}", rollup.chain_id());

    let sov_client_counter = match rollup.query(QueryReq::ClientCounter) {
        QueryResp::ClientCounter(counter) => counter,
        _ => panic!("Unexpected response"),
    };

    // TODO: this should be updated when there is a light client for sovereign chains
    let sov_client_id = ClientId::new(tm_client_type(), sov_client_counter).unwrap();

    let mut cos_builder = CosmosBuilder::default();

    let mut cos_chain = cos_builder.build_chain(InMemoryStore::default());

    wait_for_cosmos_block().await;

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

        info!("relayer: initialized manual IBC TAO layers");
    }

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

pub async fn wait_for_cosmos_block() {
    // Waits for the mock Cosmos chain to generate a few blocks before proceeding.
    sleep(Duration::from_secs(1)).await;
}
