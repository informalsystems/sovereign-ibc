use ibc_client_tendermint::types::client_type as tm_client_type;
use ibc_core::host::types::identifiers::{ClientId, Sequence};
use ibc_core::host::ValidationContext;
use sov_celestia_client::types::client_state::sov_client_type;
use sov_mock_da::MockDaService;
use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::{Context, WorkingSet};
use sov_rollup_interface::services::da::DaService;
use sov_state::{MerkleProofSpec, ProverStorage};
use tracing::info;

use super::DefaultRelayer;
use crate::configs::{default_config_with_mock_da, TestSetupConfig};
use crate::cosmos::{dummy_signer, CosmosBuilder, MockTendermint};
use crate::relayer::handle::{Handle, QueryReq, QueryResp};
use crate::relayer::relay::MockRelayer;
use crate::sovereign::{MockRollup, Runtime, DEFAULT_INIT_HEIGHT};

#[derive(Clone)]
pub struct RelayerBuilder<C, Da>
where
    C: Context,
    Da: DaService,
{
    setup_cfg: TestSetupConfig<C, Da>,
}

impl Default for RelayerBuilder<DefaultContext, MockDaService> {
    fn default() -> Self {
        Self {
            setup_cfg: default_config_with_mock_da(),
        }
    }
}

impl<C, Da> RelayerBuilder<C, Da>
where
    C: Context,
    Da: DaService,
{
    pub fn new(setup_cfg: TestSetupConfig<C, Da>) -> Self {
        Self { setup_cfg }
    }

    /// Returns the setup configuration
    pub fn setup_cfg(&self) -> &TestSetupConfig<C, Da> {
        &self.setup_cfg
    }

    /// Sets up the relayer with a manual IBC TAO
    pub fn with_manual_tao(mut self) -> Self {
        self.setup_cfg.with_manual_tao = true;
        self
    }

    /// Initializes a mock rollup and a mock Cosmos chain and sets up the relayer between them.
    pub async fn setup<S>(self) -> DefaultRelayer<C, Da, S>
    where
        C: Context<Storage = ProverStorage<S>> + Send + Sync,
        Da: DaService<Error = anyhow::Error> + Clone,
        S: MerkleProofSpec + Clone + 'static,
        <S as MerkleProofSpec>::Hasher: Send,
        MockRollup<C, Da, S>: Handle,
    {
        let runtime = Runtime::default();

        let rollup_ctx = C::new(self.setup_cfg.get_relayer_address(), DEFAULT_INIT_HEIGHT);

        let path = tempfile::tempdir().unwrap();

        let prover_storage = ProverStorage::with_path(path).unwrap();

        let mut rollup = MockRollup::new(
            runtime,
            prover_storage,
            rollup_ctx,
            MockTendermint::builder()
                .chain_id(self.setup_cfg.da_chain_id.clone())
                .build(),
            self.setup_cfg.da_service.clone(),
        );

        rollup.init(&self.setup_cfg.rollup_genesis_config()).await;

        let sov_client_counter = match rollup.query(QueryReq::ClientCounter).await {
            QueryResp::ClientCounter(counter) => counter,
            _ => panic!("Unexpected response"),
        };

        let sov_client_id = ClientId::new(sov_client_type(), sov_client_counter).unwrap();

        let mut cos_chain = CosmosBuilder::default().build();

        cos_chain.run().await;

        info!(
            "cosmos: initialized with chain id {}.",
            cos_chain.chain_id()
        );

        let cos_client_counter = cos_chain.ibc_ctx().client_counter().unwrap();

        let cos_client_id = ClientId::new(tm_client_type(), cos_client_counter).unwrap();

        if self.setup_cfg.with_manual_tao {
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

        info!("rollup: initialized with chain id {}", rollup.chain_id());

        MockRelayer::new(
            rollup.clone().into(),
            cos_chain.into(),
            sov_client_id,
            cos_client_id,
            self.setup_cfg.get_relayer_address().to_string().into(),
            dummy_signer(),
        )
    }
}
