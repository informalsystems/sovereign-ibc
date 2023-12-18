//! Contains the implementation of the Sovereign SDK rollup runner.
use std::time::Duration;

use ibc_core::host::ValidationContext;
use sov_modules_api::hooks::{FinalizeHook, SlotHooks};
use sov_modules_api::{
    Context, DispatchCall, Genesis, ModuleInfo, SlotData, StateCheckpoint, WorkingSet,
};
use sov_rollup_interface::da::BlockHeaderTrait;
use sov_rollup_interface::services::da::DaService;
use sov_state::{MerkleProofSpec, ProverStorage, Storage};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::{debug, info};

use super::{MockRollup, RuntimeCall};
use crate::sovereign::GenesisConfig;
use crate::utils::{wait_for_block, MutexUtil};

impl<C, Da, S> MockRollup<C, Da, S>
where
    C: Context<Storage = ProverStorage<S>> + Send + Sync,
    Da: DaService<Error = anyhow::Error> + Clone,
    S: MerkleProofSpec + Clone + 'static,
    <S as MerkleProofSpec>::Hasher: Send,
{
    /// Initializes the chain with the genesis configuration
    pub async fn init(
        &mut self,
        genesis_config: &GenesisConfig<C, Da::Spec>,
    ) -> StateCheckpoint<C> {
        let mut working_set = WorkingSet::new(self.prover_storage());

        self.runtime()
            .genesis(genesis_config, &mut working_set)
            .unwrap();

        self.commit(working_set.checkpoint()).await
    }

    /// Begins a block by setting the host consensus state and triggering the slot hook
    pub async fn begin_block(&mut self, checkpoint: StateCheckpoint<C>) -> StateCheckpoint<C> {
        let mut working_set = checkpoint.to_revertable();

        let current_height = self.runtime().chain_state.get_slot_height(&mut working_set);

        debug!("rollup: processing block at height {current_height}");

        let height = loop {
            // Dummy transaction to trigger the block generation
            self.da_service().send_transaction(&[0; 32]).await.unwrap();
            sleep(Duration::from_millis(100)).await;
            match self.da_service().get_last_finalized_block_header().await {
                Ok(header) => {
                    debug!("Last finalized height={}", header.height());
                    if header.height() >= current_height {
                        break current_height;
                    }
                }
                Err(err) => {
                    info!("Error receiving last finalized block header: {err:?}");
                }
            }
        };

        let block = self.da_service().get_block_at(height).await.unwrap();

        let state_root = *self.state_root().acquire_mutex();

        self.runtime().begin_slot_hook(
            block.header(),
            &block.validity_condition(),
            &state_root,
            &mut working_set,
        );

        self.set_host_consensus_state(working_set.checkpoint(), state_root)
    }

    pub async fn execute_msg(&mut self, checkpoint: StateCheckpoint<C>) -> StateCheckpoint<C> {
        let mut working_set = checkpoint.to_revertable();

        let rollup_ctx = self.rollup_ctx();

        for m in self.mempool().into_iter() {
            // Sets the sender address to the address of the 'sov-ibc'
            // module, ensuring that the module's address is used for the
            // token creation.
            if let RuntimeCall::ibc(_) = m {
                self.set_sender(self.runtime().ibc.address().clone())
            }

            self.runtime()
                .dispatch_call(m.clone(), &mut working_set, &self.rollup_ctx())
                .unwrap();

            // Resets the sender address to the address of the relayer
            self.set_sender(rollup_ctx.sender().clone());

            info!("rollup: executed message {m:?}");
        }

        *self.mempool.acquire_mutex() = vec![];

        working_set.checkpoint()
    }

    /// Commits a block by triggering the end slot hook, computing the state
    /// update and committing it to the prover storage
    pub async fn commit(&mut self, checkpoint: StateCheckpoint<C>) -> StateCheckpoint<C> {
        let checkpoint = self.begin_block(checkpoint).await;

        let checkpoint = self.execute_msg(checkpoint).await;

        let mut working_set = checkpoint.to_revertable();

        self.runtime().end_slot_hook(&mut working_set);

        let mut checkpoint = working_set.checkpoint();

        let (cache_log, witness) = checkpoint.freeze();

        let (root_hash, state_update) = self
            .prover_storage()
            .compute_state_update(cache_log, &witness)
            .expect("jellyfish merkle tree update must succeed");

        let mut working_set = checkpoint.to_revertable();

        self.runtime()
            .finalize_hook(&root_hash, &mut working_set.accessory_state());

        let mut checkpoint = working_set.checkpoint();

        let accessory_log = checkpoint.freeze_non_provable();

        self.prover_storage().commit(&state_update, &accessory_log);

        let mut working_set = checkpoint.to_revertable();

        let slot_height = self.ibc_ctx(&mut working_set).host_height().unwrap();

        *self.rollup_ctx.acquire_mutex() = C::new(
            self.rollup_ctx().sender().clone(),
            slot_height.revision_height(),
        );

        self.set_state_root(root_hash);

        working_set.checkpoint()
    }

    /// Runs the rollup chain by initializing the chain and then committing
    /// blocks at a fixed interval
    pub async fn run(&mut self) -> JoinHandle<()> {
        let mut chain = self.clone();

        let handle = tokio::task::spawn(async move {
            loop {
                let working_set = WorkingSet::new(chain.prover_storage());

                tokio::time::sleep(Duration::from_millis(200)).await;

                chain.commit(working_set.checkpoint()).await;
            }
        });

        wait_for_block().await;

        handle
    }
}
