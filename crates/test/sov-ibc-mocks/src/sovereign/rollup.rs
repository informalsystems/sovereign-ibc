//! Defines a mock rollup structure with default configurations and specs

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use ibc_client_tendermint::types::Header;
use ibc_core::client::types::Height;
use ibc_core::host::types::identifiers::ChainId;
use sov_bank::{CallMessage as BankCallMessage, Payable, TokenConfig, TokenId};
use sov_celestia_client::types::client_message::test_util::dummy_sov_header;
use sov_celestia_client::types::client_message::SovTmHeader;
use sov_consensus_state_tracker::{ConsensusStateTracker, HasConsensusState};
use sov_ibc::call::CallMessage as IbcCallMessage;
use sov_ibc::context::IbcContext;
use sov_kernels::basic::BasicKernel;
use sov_mock_da::MockFee;
use sov_modules_api::{Spec, WorkingSet};
use sov_prover_storage_manager::SimpleStorageManager;
use sov_rollup_interface::services::da::DaService;
use sov_state::{MerkleProofSpec, ProverStorage, Storage};

use super::DEFAULT_SALT;
use crate::cosmos::MockTendermint;
use crate::sovereign::runtime::RuntimeCall;
use crate::sovereign::Runtime;
use crate::utils::MutexUtil;

type Mempool<C> = Vec<RuntimeCall<C>>;

#[derive(Clone)]
pub struct MockRollup<S, Da, P>
where
    S: Spec,
    Da: DaService<Error = anyhow::Error, Fee = MockFee> + Clone,
    Da::Spec: HasConsensusState,
    P: MerkleProofSpec,
{
    kernel: ConsensusStateTracker<BasicKernel<S, Da::Spec>, S, Da::Spec>,
    runtime: Runtime<S>,
    da_service: Da,
    pub(crate) storage_manager: Arc<Mutex<SimpleStorageManager<P>>>,
    pub(crate) da_core: MockTendermint,
    pub(crate) relayer_address: S::Address,
    pub(crate) state_root: Arc<Mutex<Vec<<ProverStorage<P> as Storage>::Root>>>,
    pub(crate) mempool: Arc<Mutex<Mempool<S>>>,
}

impl<S, Da, P> MockRollup<S, Da, P>
where
    S: Spec<Storage = ProverStorage<P>> + Send + Sync,
    Da: DaService<Error = anyhow::Error, Fee = MockFee> + Clone,
    Da::Spec: HasConsensusState,
    P: MerkleProofSpec + Clone + 'static,
    <P as MerkleProofSpec>::Hasher: Send,
{
    pub fn new(
        runtime: Runtime<S>,
        storage_manager: SimpleStorageManager<P>,
        relayer_address: S::Address,
        da_core: MockTendermint,
        da_service: Da,
    ) -> Self {
        Self {
            kernel: ConsensusStateTracker::default(),
            runtime,
            da_service,
            storage_manager: Arc::new(Mutex::new(storage_manager)),
            da_core,
            relayer_address,
            state_root: Arc::new(Mutex::new(vec![])),
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

    pub fn prover_storage(&self) -> ProverStorage<P> {
        self.storage_manager.acquire_mutex().create_storage()
    }

    /// Returns the state root at a given rollup height (slot number)
    pub fn state_root(&self, height: u64) -> Option<<ProverStorage<P> as Storage>::Root> {
        self.state_root
            .acquire_mutex()
            .get(height as usize)
            .cloned()
    }

    pub fn consume_mempool(&self) -> Vec<RuntimeCall<S>> {
        self.mempool.acquire_mutex().drain(..).collect()
    }

    pub fn ibc_ctx<'a>(
        &'a self,
        working_set: &'a mut WorkingSet<S>,
    ) -> IbcContext<'a, S, WorkingSet<S>> {
        let shared_working_set = Rc::new(RefCell::new(working_set));

        IbcContext::new(&self.runtime.ibc, shared_working_set.clone())
    }

    pub fn obtain_ibc_header(&self, target_height: Height, trusted_height: Height) -> SovTmHeader {
        let blocks = self.da_core.blocks();

        let target_revision_height = target_height.revision_height();

        let height_offset = self.da_core.height() - target_revision_height;

        let da_height = target_revision_height + height_offset;

        if da_height as usize > blocks.len() {
            panic!("block index out of bounds");
        }

        let target_block = blocks[da_height as usize - 1].clone();

        let header = Header {
            signed_header: target_block.signed_header,
            validator_set: target_block.validators,
            trusted_height: trusted_height.add(height_offset),
            trusted_next_validator_set: target_block.next_validators,
        };

        let target_state_root = match self.state_root(target_revision_height - 1) {
            Some(root) => root.user_hash(),
            None => panic!("state root not found"),
        };

        dummy_sov_header(header, 1, target_revision_height, target_state_root.into())
    }

    /// Returns the balance of a user for a given token
    pub fn get_balance_of(&self, user_address: &S::Address, token_id: TokenId) -> u64 {
        let mut working_set: WorkingSet<S> = WorkingSet::new(self.prover_storage());

        self.runtime()
            .bank
            .get_balance_of(user_address.as_token_holder(), token_id, &mut working_set)
            .unwrap()
    }

    pub fn get_token_id(&self, token_config: TokenConfig<S>) -> Option<TokenId> {
        self.runtime()
            .bank
            .token_id(
                token_config.token_name,
                token_config.address_and_balances[0].0.clone(),
                DEFAULT_SALT,
            )
            .ok()
    }

    /// Searches the transfer module by given token denom and returns the token
    /// ID if the token has been minted.
    pub fn get_minted_token_id(&self, token_denom: String) -> Option<TokenId> {
        let mut working_set = WorkingSet::new(self.prover_storage());

        self.runtime()
            .ibc_transfer
            .minted_token_id(token_denom, &mut working_set)
            .map(|token| token.token_id)
            .ok()
    }

    pub(crate) fn push_state_root(&mut self, state_root: <ProverStorage<P> as Storage>::Root) {
        let mut state_roots = self.state_root.acquire_mutex();

        state_roots.push(state_root);
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
