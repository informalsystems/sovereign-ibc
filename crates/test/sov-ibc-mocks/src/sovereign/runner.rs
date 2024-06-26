//! Contains the implementation of the Sovereign SDK rollup runner.
use std::time::Duration;

use sov_consensus_state_tracker::HasConsensusState;
use sov_mock_da::MockFee;
use sov_modules_api::runtime::capabilities::{Kernel, KernelSlotHooks};
use sov_modules_api::{
    CallResponse, Context, DispatchCall, Gas, Genesis, KernelWorkingSet, SlotData, Spec,
    StateCheckpoint,
};
use sov_rollup_interface::da::BlockHeaderTrait;
use sov_rollup_interface::services::da::DaService;
use sov_state::storage::StateUpdate;
use sov_state::{MerkleProofSpec, ProverStorage, Storage};
use tokio::task::JoinHandle;
use tracing::{debug, info};

use super::MockRollup;
use crate::configs::TestSetupConfig;
use crate::utils::{wait_for_block, MutexUtil};

impl<S, Da, P> MockRollup<S, Da, P>
where
    S: Spec<Storage = ProverStorage<P>> + Send + Sync,
    Da: DaService<Error = anyhow::Error, Fee = MockFee> + Clone,
    Da::Spec: HasConsensusState,
    P: MerkleProofSpec + Clone + 'static,
    <P as MerkleProofSpec>::Hasher: Send,
{
    /// Initializes the chain with the genesis configuration
    pub fn init(&mut self, setup_cfg: &TestSetupConfig<S, Da>) {
        self.da_core
            .advance_da_block_up_to(setup_cfg.genesis_da_height);

        let mut checkpoint = StateCheckpoint::new(self.prover_storage());

        let mut kernel_working_set = KernelWorkingSet::uninitialized(&mut checkpoint);

        self.kernel()
            .genesis(&setup_cfg.kernel_genesis_config(), &mut kernel_working_set)
            .unwrap();

        let mut working_set = checkpoint.to_revertable_unmetered();

        self.runtime()
            .genesis(&setup_cfg.runtime_genesis_config(), &mut working_set)
            .unwrap();

        self.commit(working_set.checkpoint().0);
    }

    /// Begins processing a DA block by triggering the `begin_slot_hook`
    pub async fn begin_slot(&mut self, mut checkpoint: StateCheckpoint<S>) -> StateCheckpoint<S> {
        let kernel_working_set = KernelWorkingSet::from_kernel(self.kernel(), &mut checkpoint);

        let current_height = kernel_working_set.current_slot();

        let pre_state_root = self.state_root(current_height).unwrap();

        let height = loop {
            self.da_core.grow_blocks(pre_state_root.as_ref().to_vec());
            // Dummy transaction to trigger the block generation
            self.da_service()
                .send_transaction(&[0; 32], MockFee::zero())
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;
            match self.da_service().get_last_finalized_block_header().await {
                Ok(header) => {
                    debug!("Last finalized height={}", header.height());
                    if header.height() >= current_height {
                        break header.height();
                    }
                }
                Err(err) => {
                    info!("Error receiving last finalized block header: {err:?}");
                }
            }
        };

        let block = self.da_service().get_block_at(height).await.unwrap();

        self.kernel().begin_slot_hook(
            block.header(),
            &block.validity_condition(),
            &pre_state_root,
            &mut checkpoint,
        );

        checkpoint
    }

    /// Apply a slot by executing the messages in the mempool and committing the state update
    pub async fn apply_slot(&mut self, checkpoint: StateCheckpoint<S>) {
        let mut checkpoint = self.execute_msg(checkpoint);

        self.kernel().end_slot_hook(&Gas::zero(), &mut checkpoint);

        self.commit(checkpoint);
    }

    pub fn execute_msg(&mut self, mut checkpoint: StateCheckpoint<S>) -> StateCheckpoint<S> {
        let kernel_working_set = KernelWorkingSet::from_kernel(self.kernel(), &mut checkpoint);

        let visible_slot = kernel_working_set.virtual_slot();

        let mut working_set = checkpoint.to_revertable_unmetered();

        // create the Rollup context dynamically
        // using the relayer address and the visible slot
        let rollup_ctx = Context::new(
            self.relayer_address.clone(),
            Default::default(),
            self.relayer_address.clone(),
            visible_slot,
        );

        for m in self.consume_mempool() {
            // NOTE: on failures, we silently ignore the message and continue as
            // it is in the real-case scenarios
            self.runtime()
                .dispatch_call(m.clone(), &mut working_set, &rollup_ctx)
                .unwrap_or_else(|e| {
                    info!("rollup: error executing message: {e:?}");
                    CallResponse::default()
                });
        }

        working_set.checkpoint().0
    }

    /// Commits the state update to the prover storage
    pub fn commit(&mut self, checkpoint: StateCheckpoint<S>) {
        let (cache_log, accessory_delta, witness) = checkpoint.freeze();

        let (root_hash, mut state_update) = self
            .prover_storage()
            .compute_state_update(cache_log, &witness)
            .expect("jellyfish merkle tree update must succeed");

        state_update.add_accessory_items(accessory_delta.freeze());

        let change_set = self.prover_storage().materialize_changes(&state_update);

        self.storage_manager.acquire_mutex().commit(change_set);

        self.push_state_root(root_hash);
    }

    /// Runs the rollup by initializing the rollup and then committing blocks at
    /// a fixed interval
    pub async fn run(&mut self) -> JoinHandle<()> {
        let mut chain = self.clone();

        let handle = tokio::task::spawn(async move {
            loop {
                let checkpoint = StateCheckpoint::new(chain.prover_storage());

                let checkpoint = chain.begin_slot(checkpoint).await;

                chain.apply_slot(checkpoint).await;
            }
        });

        wait_for_block().await;

        handle
    }
}
