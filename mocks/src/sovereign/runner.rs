//! Contains the implementation of the Sovereign SDK rollup runner.
use std::time::Duration;

use sov_modules_api::hooks::{FinalizeHook, SlotHooks};
use sov_modules_api::runtime::capabilities::{Kernel, KernelSlotHooks};
use sov_modules_api::{
    DispatchCall, Gas, Genesis, KernelWorkingSet, ModuleInfo, SlotData, Spec, StateCheckpoint,
    VersionedStateReadWriter,
};
use sov_modules_stf_blueprint::kernels::basic::BasicKernelGenesisConfig;
use sov_rollup_interface::da::BlockHeaderTrait;
use sov_rollup_interface::services::da::DaService;
use sov_state::{MerkleProofSpec, ProverStorage, Storage};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::{debug, info};

use super::{GenesisConfig, MockRollup, RuntimeCall};
use crate::utils::{wait_for_block, MutexUtil};

impl<S, Da, P> MockRollup<S, Da, P>
where
    S: Spec<Storage = ProverStorage<P>> + Send + Sync,
    Da: DaService<Error = anyhow::Error> + Clone,
    P: MerkleProofSpec + Clone + 'static,
    <P as MerkleProofSpec>::Hasher: Send,
{
    /// Initializes the chain with the genesis configuration
    pub async fn init(
        &mut self,
        kernel_genesis_config: &BasicKernelGenesisConfig<S, Da::Spec>,
        runtime_genesis_config: &GenesisConfig<S, Da::Spec>,
    ) -> StateCheckpoint<S> {
        let mut checkpoint = StateCheckpoint::new(self.prover_storage());

        let mut kernel_working_set = KernelWorkingSet::uninitialized(&mut checkpoint);

        self.kernel()
            .genesis(kernel_genesis_config, &mut kernel_working_set)
            .unwrap();

        let mut working_set = checkpoint.to_revertable(Default::default());

        self.runtime()
            .genesis(runtime_genesis_config, &mut working_set)
            .unwrap();

        let checkpoint = working_set.checkpoint().0;

        let checkpoint = self.begin_block(checkpoint).await;

        self.commit(checkpoint).await
    }

    /// Begins a block by setting the host consensus state and triggering the slot hook
    pub async fn begin_block(&mut self, mut checkpoint: StateCheckpoint<S>) -> StateCheckpoint<S> {
        let current_height = self.rollup_ctx.acquire_mutex().visible_slot_number();

        debug!("rollup: processing block at height {current_height}");

        let height = loop {
            self.da_core
                .grow_blocks(self.state_root.lock().unwrap().as_ref().to_vec());
            // Dummy transaction to trigger the block generation
            self.da_service().send_transaction(&[0; 32]).await.unwrap();
            sleep(Duration::from_millis(100)).await;
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

        let state_root = *self.state_root().acquire_mutex();

        self.kernel().begin_slot_hook(
            block.header(),
            &block.validity_condition(),
            &state_root,
            &mut checkpoint,
        );

        let kernel_working_set = KernelWorkingSet::from_kernel(self.kernel(), &mut checkpoint);
        let mut versioned_working_set =
            VersionedStateReadWriter::from_kernel_ws_virtual(kernel_working_set);

        let visible_hash = <S as Spec>::VisibleHash::from(state_root);

        self.runtime()
            .begin_slot_hook(visible_hash, &mut versioned_working_set);

        let mut working_set = checkpoint.to_revertable(Default::default());

        self.set_host_consensus_state(state_root, &mut working_set);

        working_set.checkpoint().0
    }

    pub async fn execute_msg(&mut self, checkpoint: StateCheckpoint<S>) -> StateCheckpoint<S> {
        let mut working_set = checkpoint.to_revertable(Default::default());

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
        }

        *self.mempool.acquire_mutex() = vec![];

        working_set.checkpoint().0
    }

    /// Commits a block by triggering the end slot hook, computing the state
    /// update and committing it to the prover storage
    pub async fn commit(&mut self, checkpoint: StateCheckpoint<S>) -> StateCheckpoint<S> {
        let mut checkpoint = self.execute_msg(checkpoint).await;

        self.kernel().end_slot_hook(&Gas::zero(), &mut checkpoint);

        self.runtime().end_slot_hook(&mut checkpoint);

        let (cache_log, witness) = checkpoint.freeze();

        let (root_hash, state_update) = self
            .prover_storage()
            .compute_state_update(cache_log, &witness)
            .expect("jellyfish merkle tree update must succeed");

        let visible_root_hash = <S as Spec>::VisibleHash::from(root_hash);

        self.runtime()
            .finalize_hook(visible_root_hash, &mut checkpoint.accessory_state());

        let accessory_log = checkpoint.freeze_non_provable();

        self.prover_storage().commit(&state_update, &accessory_log);

        self.set_state_root(root_hash);

        checkpoint
    }

    /// Runs the rollup chain by initializing the chain and then committing
    /// blocks at a fixed interval
    pub async fn run(&mut self) -> JoinHandle<()> {
        let mut chain = self.clone();

        let handle = tokio::task::spawn(async move {
            loop {
                let checkpoint = StateCheckpoint::new(chain.prover_storage());
                let checkpoint = chain.begin_block(checkpoint).await;

                tokio::time::sleep(Duration::from_millis(200)).await;

                chain.commit(checkpoint).await;
            }
        });

        wait_for_block().await;

        handle
    }
}
