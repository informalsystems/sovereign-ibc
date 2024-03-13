//! Defines a mock rollup structure with default configurations and specs

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::CommitmentRoot;
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::host::ValidationContext;
use jmt::RootHash;
use sov_bank::CallMessage as BankCallMessage;
use sov_celestia_client::types::consensus_state::{SovTmConsensusState, TmConsensusParams};
use sov_consensus_state_tracker::{ConsensusStateTracker, HasConsensusState};
use sov_ibc::call::CallMessage as IbcCallMessage;
use sov_ibc::clients::AnyConsensusState;
use sov_ibc::context::IbcContext;
use sov_modules_api::{Context, DaSpec, Spec, WorkingSet};
use sov_modules_stf_blueprint::kernels::basic::BasicKernel;
use sov_rollup_interface::services::da::DaService;
use sov_state::{MerkleProofSpec, ProverStorage, Storage, StorageRoot};
use tendermint::{Hash, Time};

use crate::cosmos::MockTendermint;
use crate::sovereign::runtime::RuntimeCall;
use crate::sovereign::Runtime;
use crate::utils::MutexUtil;

type Mempool<C, Da> = Vec<RuntimeCall<C, Da>>;

#[derive(Clone)]
pub struct MockRollup<S, Da, P>
where
    S: Spec,
    Da: DaService<Error = anyhow::Error> + Clone,
    P: MerkleProofSpec,
{
    kernel: BasicKernel<S, Da::Spec>,
    runtime: Runtime<S, Da::Spec>,
    da_service: Da,
    prover_storage: ProverStorage<P>,
    pub(crate) da_core: MockTendermint,
    pub(crate) rollup_ctx: Arc<Mutex<Context<S>>>,
    pub(crate) state_root: Arc<Mutex<<ProverStorage<P> as Storage>::Root>>,
    pub(crate) mempool: Arc<Mutex<Mempool<S, Da::Spec>>>,
}

impl<S: Spec, Da: DaSpec> Clone for RuntimeCall<S, Da> {
    fn clone(&self) -> Self {
        match self {
            RuntimeCall::bank(call) => RuntimeCall::bank(call.clone()),
            RuntimeCall::ibc(call) => RuntimeCall::ibc(call.clone()),
            RuntimeCall::ibc_transfer(_) => RuntimeCall::ibc_transfer(()),
        }
    }
}

impl<S: Spec, Da: DaSpec> From<IbcCallMessage> for RuntimeCall<S, Da> {
    fn from(call: IbcCallMessage) -> Self {
        RuntimeCall::ibc(call)
    }
}

impl<S: Spec, Da: DaSpec> From<BankCallMessage<S>> for RuntimeCall<S, Da> {
    fn from(call: BankCallMessage<S>) -> Self {
        RuntimeCall::bank(call)
    }
}

impl<S, Da, P> MockRollup<S, Da, P>
where
    S: Spec<Storage = ProverStorage<P>> + Send + Sync,
    Da: DaService<Error = anyhow::Error> + Clone,
    P: MerkleProofSpec + Clone + 'static,
    <P as MerkleProofSpec>::Hasher: Send,
{
    pub fn new(
        runtime: Runtime<S, Da::Spec>,
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

    pub fn kernel(&self) -> &BasicKernel<S, Da::Spec> {
        &self.kernel
    }

    pub fn runtime(&self) -> &Runtime<S, Da::Spec> {
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

    pub fn mempool(&self) -> Vec<RuntimeCall<S, Da::Spec>> {
        self.mempool.acquire_mutex().clone()
    }

    pub fn ibc_ctx<'a>(
        &'a self,
        working_set: &'a mut WorkingSet<S>,
    ) -> IbcContext<'a, S, Da::Spec> {
        let shared_working_set = Rc::new(RefCell::new(working_set));

        IbcContext::new(
            &self.runtime.ibc,
            Some(self.rollup_ctx.acquire_mutex().clone()),
            shared_working_set.clone(),
        )
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

    /// Sets the host consensus state when processing each block
    pub(crate) fn set_host_consensus_state(
        &mut self,
        root_hash: <ProverStorage<P> as Storage>::Root,
        working_set: &mut WorkingSet<S>,
    ) {
        let mut ibc_ctx = self.ibc_ctx(working_set);

        let current_height = ibc_ctx
            .host_height()
            .unwrap_or(Height::new(0, 1).expect("valid height"));

        let visible_hash = <S as Spec>::VisibleHash::from(root_hash);

        let sov_consensus_state = SovTmConsensusState::new(
            CommitmentRoot::from_bytes(&visible_hash.into()),
            TmConsensusParams::new(
                Time::now(),
                Hash::Sha256([
                    0xd6, 0xb9, 0x39, 0x22, 0xc3, 0x3a, 0xae, 0xbe, 0xc9, 0x4, 0x35, 0x66, 0xcb,
                    0x4b, 0x1b, 0x48, 0x36, 0x5b, 0x13, 0x58, 0xb6, 0x7c, 0x7d, 0xef, 0x98, 0x6d,
                    0x9e, 0xe1, 0x86, 0x1b, 0xc1, 0x43,
                ]),
            ),
        )
        .into();

        let consensus_state = AnyConsensusState::Sovereign(sov_consensus_state);

        ibc_ctx
            .store_host_consensus_state(current_height, consensus_state)
            .unwrap();
    }
}
