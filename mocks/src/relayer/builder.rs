use ibc_client_tendermint::types::client_type as tm_client_type;
use ibc_core::host::types::identifiers::Sequence;
use ibc_core::host::ValidationContext;
use sov_celestia_client::types::client_state::sov_client_type;
#[cfg(all(feature = "celestia-da", not(feature = "mock-da")))]
use sov_consensus_state_tracker::CelestiaService;
use sov_consensus_state_tracker::HasConsensusState;
#[cfg(feature = "mock-da")]
use sov_consensus_state_tracker::MockDaService;
use sov_modules_api::{Context, Spec, WorkingSet};
use sov_prover_storage_manager::new_orphan_storage;
use sov_rollup_interface::services::da::DaService;
use sov_state::{MerkleProofSpec, ProverStorage};
use tracing::info;

use super::DefaultRelayer;
#[cfg(all(feature = "celestia-da", not(feature = "mock-da")))]
use crate::configs::default_config_with_celestia_da;
#[cfg(feature = "mock-da")]
use crate::configs::default_config_with_mock_da;
use crate::configs::{DefaultSpec, TestSetupConfig};
use crate::cosmos::{dummy_signer, CosmosBuilder, MockTendermint};
use crate::relayer::handle::{Handle, QueryReq, QueryResp};
use crate::relayer::relay::MockRelayer;
use crate::sovereign::{MockRollup, Runtime, DEFAULT_INIT_HEIGHT};

#[derive(Clone)]
pub struct RelayerBuilder<S, Da>
where
    S: Spec,
    Da: DaService,
{
    setup_cfg: TestSetupConfig<S, Da>,
}

#[cfg(feature = "mock-da")]
impl RelayerBuilder<DefaultSpec, MockDaService> {
    pub async fn default() -> Self {
        Self {
            setup_cfg: default_config_with_mock_da(),
        }
    }
}

#[cfg(all(feature = "celestia-da", not(feature = "mock-da")))]
impl RelayerBuilder<DefaultSpec, CelestiaService> {
    pub async fn default() -> Self {
        Self {
            setup_cfg: default_config_with_celestia_da().await,
        }
    }
}

impl<S, Da> RelayerBuilder<S, Da>
where
    S: Spec,
    Da: DaService,
{
    pub fn new(setup_cfg: TestSetupConfig<S, Da>) -> Self {
        Self { setup_cfg }
    }

    /// Returns the setup configuration
    pub fn setup_cfg(&self) -> &TestSetupConfig<S, Da> {
        &self.setup_cfg
    }

    /// Sets up the relayer with a manual IBC TAO
    pub fn with_manual_tao(mut self) -> Self {
        self.setup_cfg.with_manual_tao = true;
        self
    }

    /// Initializes a mock rollup and a mock Cosmos chain and sets up the relayer between them.
    pub async fn setup<P>(self) -> DefaultRelayer<S, Da, P>
    where
        S: Spec<Storage = ProverStorage<P>> + Send + Sync,
        Da: DaService<Error = anyhow::Error> + Clone,
        Da::Spec: HasConsensusState,
        P: MerkleProofSpec + Clone + 'static,
        <P as MerkleProofSpec>::Hasher: Send,
        MockRollup<S, Da, P>: Handle,
    {
        let runtime = Runtime::default();

        let sender_address = self.setup_cfg.get_relayer_address();

        let rollup_ctx = Context::new(sender_address.clone(), sender_address, DEFAULT_INIT_HEIGHT);

        let tmpdir = tempfile::tempdir().unwrap();

        let prover_storage = new_orphan_storage(tmpdir.path()).unwrap();

        let mut rollup = MockRollup::new(
            runtime,
            prover_storage,
            rollup_ctx,
            MockTendermint::builder()
                .chain_id(self.setup_cfg.da_chain_id.clone())
                .build(),
            self.setup_cfg.da_service.clone(),
        );

        rollup
            .init(
                &self.setup_cfg.kernel_genesis_config(),
                &self.setup_cfg.runtime_genesis_config(),
            )
            .await;

        let sov_client_counter = match rollup.query(QueryReq::ClientCounter).await {
            QueryResp::ClientCounter(counter) => counter,
            _ => panic!("Unexpected response"),
        };

        let sov_client_id = sov_client_type().build_client_id(sov_client_counter);

        let mut cos_chain = CosmosBuilder::default().build();

        cos_chain.run().await;

        info!(
            "cosmos: initialized with chain id {}.",
            cos_chain.chain_id()
        );

        rollup.run().await;

        info!("rollup: initialized with chain id {}", rollup.chain_id());

        let cos_client_counter = cos_chain.ibc_ctx().client_counter().unwrap();

        let cos_client_id = tm_client_type().build_client_id(cos_client_counter);

        if self.setup_cfg.with_manual_tao {
            let cos_client_id = rollup.setup_client(cos_chain.chain_id()).await;
            let sov_client_id = cos_chain.setup_client(rollup.chain_id());

            let sov_conn_id = rollup
                .setup_connection(cos_client_id, cos_chain.ibc_ctx().commitment_prefix())
                .await;

            let mut working_set = WorkingSet::new(rollup.prover_storage());
            let cos_conn_id = cos_chain.setup_connection(
                sov_client_id,
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
