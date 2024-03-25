//! Defines a mock rollup structure with default configurations and specs

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use ibc_core::host::types::identifiers::ChainId;
use jmt::RootHash;
use sov_bank::CallMessage as BankCallMessage;
use sov_consensus_state_tracker::{ConsensusStateTracker, HasConsensusState};
use sov_ibc::call::CallMessage as IbcCallMessage;
use sov_ibc::context::IbcContext;
use sov_modules_api::{Context, Spec, WorkingSet};
use sov_modules_stf_blueprint::kernels::basic::BasicKernel;
use sov_rollup_interface::services::da::DaService;
use sov_state::{MerkleProofSpec, ProverStorage, Storage, StorageRoot};

use crate::cosmos::MockTendermint;
use crate::sovereign::runtime::RuntimeCall;
use crate::sovereign::Runtime;
use crate::utils::MutexUtil;

type Mempool<C> = Vec<RuntimeCall<C>>;

#[derive(Clone)]
pub struct MockRollup<S, Da, P>
where
    S: Spec,
    Da: DaService<Error = anyhow::Error> + Clone,
    Da::Spec: HasConsensusState,
    P: MerkleProofSpec,
{
    kernel: ConsensusStateTracker<BasicKernel<S, Da::Spec>, S, Da::Spec>,
    runtime: Runtime<S>,
    da_service: Da,
    prover_storage: ProverStorage<P>,
    pub(crate) da_core: MockTendermint,
    pub(crate) rollup_ctx: Arc<Mutex<Context<S>>>,
    pub(crate) state_root: Arc<Mutex<<ProverStorage<P> as Storage>::Root>>,
    pub(crate) mempool: Arc<Mutex<Mempool<S>>>,
}

impl<S, Da, P> MockRollup<S, Da, P>
where
    S: Spec<Storage = ProverStorage<P>> + Send + Sync,
    Da: DaService<Error = anyhow::Error> + Clone,
    Da::Spec: HasConsensusState,
    P: MerkleProofSpec + Clone + 'static,
    <P as MerkleProofSpec>::Hasher: Send,
{
    pub fn new(
        runtime: Runtime<S>,
        prover_storage: ProverStorage<P>,
        rollup_ctx: Context<S>,
        da_core: MockTendermint,
        da_service: Da,
    ) -> Self {
        Self {
            kernel: ConsensusStateTracker::default(),
            runtime,
            da_service,
            prover_storage,
            da_core,
            rollup_ctx: Arc::new(Mutex::new(rollup_ctx)),
            state_root: Arc::new(Mutex::new(StorageRoot::new(
                RootHash([1; 32]),
                RootHash([0; 32]),
            ))),
            mempool: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn chain_id(&self) -> &ChainId {
        self.da_core.chain_id()
    }

    pub fn kernel(&self) -> &ConsensusStateTracker<BasicKernel<S, Da::Spec>, S, Da::Spec> {
        &self.kernel
    }

    pub fn runtime(&self) -> &Runtime<S> {
        &self.runtime
    }

    pub fn da_service(&self) -> &Da {
        &self.da_service
    }

    pub fn rollup_ctx(&self) -> Context<S> {
        self.rollup_ctx.acquire_mutex().clone()
    }

    pub fn prover_storage(&self) -> ProverStorage<P> {
        self.prover_storage.clone()
    }

    pub fn state_root(&self) -> Arc<Mutex<<ProverStorage<P> as Storage>::Root>> {
        self.state_root.clone()
    }

    pub fn mempool(&self) -> Vec<RuntimeCall<S>> {
        self.mempool.acquire_mutex().clone()
    }

    pub fn ibc_ctx<'a>(&'a self, working_set: &'a mut WorkingSet<S>) -> IbcContext<'a, S> {
        let shared_working_set = Rc::new(RefCell::new(working_set));

        IbcContext::new(&self.runtime.ibc, shared_working_set.clone())
    }

    /// Returns the balance of a user for a given token
    pub fn get_balance_of(&self, user_address: S::Address, token_address: S::Address) -> u64 {
        let mut working_set: WorkingSet<S> = WorkingSet::new(self.prover_storage());

        self.runtime()
            .bank
            .get_balance_of(user_address, token_address, &mut working_set)
            .unwrap()
    }

    /// Returns token address of an IBC denom
    pub fn get_minted_token_address(&self, token_denom: String) -> Option<S::Address> {
        let mut working_set = WorkingSet::new(self.prover_storage());

        self.runtime()
            .ibc_transfer
            .minted_token(token_denom, &mut working_set)
            .map(|token| token.address)
            .ok()
    }

    /// Searches the transfer module to retrieve the address of the token held
    /// in escrow, based on its token denom.
    pub fn get_escrowed_token_address(&self, token_denom: String) -> Option<S::Address> {
        let mut working_set = WorkingSet::new(self.prover_storage());

        self.runtime()
            .ibc_transfer
            .escrowed_token(token_denom, &mut working_set)
            .map(|token| token.address)
            .ok()
    }

    pub(crate) fn set_state_root(&mut self, state_root: <ProverStorage<P> as Storage>::Root) {
        *self.state_root.acquire_mutex() = state_root;
    }

    pub(crate) fn resolve_ctx(&mut self, sender: S::Address, height: u64) {
        *self.rollup_ctx.acquire_mutex() = Context::new(sender.clone(), sender, height);
    }
}

impl<S: Spec> Clone for RuntimeCall<S> {
    fn clone(&self) -> Self {
        match self {
            RuntimeCall::bank(call) => RuntimeCall::bank(call.clone()),
            RuntimeCall::ibc(call) => RuntimeCall::ibc(call.clone()),
            RuntimeCall::ibc_transfer(_) => RuntimeCall::ibc_transfer(()),
        }
    }
}

impl<S: Spec> From<IbcCallMessage> for RuntimeCall<S> {
    fn from(call: IbcCallMessage) -> Self {
        RuntimeCall::ibc(call)
    }
}

impl<S: Spec> From<BankCallMessage<S>> for RuntimeCall<S> {
    fn from(call: BankCallMessage<S>) -> Self {
        RuntimeCall::bank(call)
    }
}
