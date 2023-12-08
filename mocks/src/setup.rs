use std::time::Duration;

use basecoin_store::impls::InMemoryStore;
use ibc_client_tendermint::types::client_type as tm_client_type;
use ibc_core::host::types::identifiers::{ChainId, ClientId, Sequence};
use ibc_core::host::ValidationContext;
use sov_mock_da::{MockAddress, MockDaService};
use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::{Context, WorkingSet};
use sov_state::{DefaultStorageSpec, ProverStorage};
use tokio::time::sleep;

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

    let relayer_address = config.bank_config.tokens[0]
        .address_and_balances
        .last()
        .unwrap();

    let runtime = Runtime::default();

    let rollup_ctx = DefaultContext::new(relayer_address.0, 0);

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

    let sov_client_counter = match rollup.query(QueryReq::ClientCounter) {
        QueryResp::ClientCounter(counter) => counter,
        _ => panic!("Unexpected response"),
    };

    // TODO: this should be updated when there is a light client for sovereign chains
    let sov_client_id = ClientId::new(tm_client_type(), sov_client_counter).unwrap();

    let mut cos_builder = CosmosBuilder::default();

    let mut cos_handler = cos_builder.build_chain(InMemoryStore::default());

    // Waits for the mock Cosmos chain to generate a few blocks before proceeding.
    sleep(Duration::from_secs(1)).await;

    let cos_client_counter = cos_handler.ibc_ctx().client_counter().unwrap();

    let cos_client_id = ClientId::new(tm_client_type(), cos_client_counter).unwrap();

    if with_manual_tao {
        let sov_client_id = rollup.setup_client(cos_handler.chain_id()).await;
        let cos_client_id = cos_handler.setup_client(rollup.chain_id());

        let sov_conn_id = rollup
            .setup_connection(sov_client_id, cos_handler.ibc_ctx().commitment_prefix())
            .await;

        let mut working_set = WorkingSet::new(rollup.prover_storage());
        let cos_conn_id = cos_handler.setup_connection(
            cos_client_id,
            rollup.ibc_ctx(&mut working_set).commitment_prefix(),
        );

        let (sov_port_id, sov_chan_id) = rollup.setup_channel(sov_conn_id).await;
        let (cos_port_id, cos_chan_id) = cos_handler.setup_channel(cos_conn_id);

        rollup
            .with_send_sequence(sov_port_id, sov_chan_id, Sequence::from(1))
            .await;
        cos_handler.with_send_sequence(cos_port_id, cos_chan_id, Sequence::from(1));
    }

    (
        MockRelayer::new(
            rollup.clone().into(),
            cos_handler.into(),
            sov_client_id,
            cos_client_id,
            dummy_signer(),
            dummy_signer(),
        ),
        rollup,
    )
}
