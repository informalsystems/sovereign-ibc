//! Defines a mock rollup structure with default configurations and specs

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use ibc_client_tendermint::types::ConsensusState as TmConsensusState;
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::CommitmentRoot;
use ibc_core::host::types::identifiers::ChainId;
use sov_bank::{get_token_address, CallMessage as BankCallMessage, TokenConfig};
use sov_ibc::call::CallMessage as IbcCallMessage;
use sov_ibc::clients::AnyConsensusState;
use sov_ibc::context::IbcContext;
use sov_modules_api::{Context, DaSpec, StateCheckpoint, WorkingSet};
use sov_rollup_interface::services::da::DaService;
use sov_state::{MerkleProofSpec, ProverStorage, Storage};
use tendermint::{Hash, Time};

use crate::sovereign::runtime::RuntimeCall;
use crate::sovereign::{Runtime, RuntimeConfig};

type Mempool<C, Da> = Vec<RuntimeCall<C, Da>>;

#[derive(Clone)]
pub struct MockRollup<C, Da, S>
where
    C: Context,
    Da: DaService<Error = anyhow::Error> + Clone,
    S: MerkleProofSpec,
{
    chain_id: ChainId,
    config: RuntimeConfig<C>,
    runtime: Runtime<C, Da::Spec>,
    da_service: Da,
    prover_storage: ProverStorage<S>,
    pub(crate) rollup_ctx: Arc<Mutex<C>>,
    pub(crate) state_root: Arc<Mutex<<ProverStorage<S> as Storage>::Root>>,
    pub(crate) mempool: Arc<Mutex<Mempool<C, Da::Spec>>>,
}

impl<C: Context, Da: DaSpec> Clone for RuntimeCall<C, Da> {
    fn clone(&self) -> Self {
        match self {
            RuntimeCall::bank(call) => RuntimeCall::bank(call.clone()),
            RuntimeCall::chain_state(_) => RuntimeCall::chain_state(()),
            RuntimeCall::ibc(call) => RuntimeCall::ibc(call.clone()),
            RuntimeCall::ibc_transfer(_) => RuntimeCall::ibc_transfer(()),
        }
    }
}

impl<C: Context, Da: DaSpec> From<IbcCallMessage> for RuntimeCall<C, Da> {
    fn from(call: IbcCallMessage) -> Self {
        RuntimeCall::ibc(call)
    }
}

impl<C: Context, Da: DaSpec> From<BankCallMessage<C>> for RuntimeCall<C, Da> {
    fn from(call: BankCallMessage<C>) -> Self {
        RuntimeCall::bank(call)
    }
}

impl<C, Da, S> MockRollup<C, Da, S>
where
    C: Context<Storage = ProverStorage<S>> + Send + Sync,
    Da: DaService<Error = anyhow::Error> + Clone,
    S: MerkleProofSpec + Clone + 'static,
    <S as MerkleProofSpec>::Hasher: Send,
{
    pub fn new(
        chain_id: ChainId,
        config: RuntimeConfig<C>,
        runtime: Runtime<C, Da::Spec>,
        prover_storage: ProverStorage<S>,
        rollup_ctx: C,
        da_service: Da,
    ) -> Self {
        Self {
            chain_id,
            config,
            runtime,
            da_service,
            prover_storage,
            rollup_ctx: Arc::new(Mutex::new(rollup_ctx)),
            state_root: Arc::new(Mutex::new(jmt::RootHash([0; 32]))),
            mempool: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    pub fn rollup_ctx(&self) -> C {
        self.rollup_ctx.lock().unwrap().clone()
    }

    pub fn config(&self) -> &RuntimeConfig<C> {
        &self.config
    }

    pub fn runtime(&self) -> &Runtime<C, Da::Spec> {
        &self.runtime
    }

    pub fn da_service(&self) -> Da {
        self.da_service.clone()
    }

    pub fn prover_storage(&self) -> ProverStorage<S> {
        self.prover_storage.clone()
    }

    pub fn state_root(&self) -> Arc<Mutex<<ProverStorage<S> as Storage>::Root>> {
        self.state_root.clone()
    }

    pub fn mempool(&self) -> Vec<RuntimeCall<C, Da::Spec>> {
        self.mempool.lock().unwrap().clone()
    }

    pub fn ibc_ctx<'a>(
        &'a self,
        working_set: &'a mut WorkingSet<C>,
    ) -> IbcContext<'a, C, Da::Spec> {
        let shared_working_set = Rc::new(RefCell::new(working_set));

        IbcContext::new(&self.runtime.ibc, shared_working_set.clone())
    }

    /// Returns list of tokens in the bank configuration
    pub fn get_tokens(&self) -> &Vec<TokenConfig<C>> {
        &self.config.bank_config.tokens
    }

    /// Returns the token address for a given token configuration
    pub fn get_token_address(&self, token_cfg: &TokenConfig<C>) -> C::Address {
        get_token_address::<C>(
            &token_cfg.token_name,
            self.get_relayer_address().as_ref(),
            token_cfg.salt,
        )
    }

    /// Returns the address of the relayer. We use the last address in the list
    /// as the relayer address
    pub fn get_relayer_address(&self) -> C::Address {
        self.config.bank_config.tokens[0]
            .address_and_balances
            .last()
            .unwrap()
            .0
            .clone()
    }

    /// Returns the balance of a user for a given token
    pub fn get_balance_of(&self, user_address: C::Address, token_address: C::Address) -> u64 {
        let mut working_set = WorkingSet::new(self.prover_storage());

        self.runtime()
            .bank
            .get_balance_of(user_address, token_address, &mut working_set)
            .unwrap()
    }

    /// Returns token address of an IBC denom
    pub fn get_minted_token_address(&self, token_denom: String) -> Option<C::Address> {
        let mut working_set = WorkingSet::new(self.prover_storage());

        self.runtime()
            .ibc_transfer
            .minted_token(token_denom, &mut working_set)
            .map(|token| token.address)
            .ok()
    }

    /// Searches the transfer module to retrieve the address of the token held
    /// in escrow, based on its token denom.
    pub fn get_escrowed_token_address(&self, token_denom: String) -> Option<C::Address> {
        let mut working_set = WorkingSet::new(self.prover_storage());

        self.runtime()
            .ibc_transfer
            .escrowed_token(token_denom, &mut working_set)
            .map(|token| token.address)
            .ok()
    }

    pub(crate) fn set_state_root(&mut self, state_root: <ProverStorage<S> as Storage>::Root) {
        *self.state_root.lock().unwrap() = state_root;
    }

    /// Sets the host consensus state when processing each block
    pub(crate) fn set_host_consensus_state(
        &mut self,
        checkpoint: StateCheckpoint<C>,
        root_hash: <ProverStorage<S> as Storage>::Root,
    ) -> StateCheckpoint<C> {
        let mut working_set = checkpoint.to_revertable();

        let current_height = self.runtime().chain_state.get_slot_height(&mut working_set);

        let consensus_state = AnyConsensusState::Tendermint(
            TmConsensusState::new(
                CommitmentRoot::from_bytes(&root_hash.0),
                Time::now(),
                Hash::Sha256([
                    0xd6, 0xb9, 0x39, 0x22, 0xc3, 0x3a, 0xae, 0xbe, 0xc9, 0x4, 0x35, 0x66, 0xcb,
                    0x4b, 0x1b, 0x48, 0x36, 0x5b, 0x13, 0x58, 0xb6, 0x7c, 0x7d, 0xef, 0x98, 0x6d,
                    0x9e, 0xe1, 0x86, 0x1b, 0xc1, 0x43,
                ]),
            )
            .into(),
        );

        self.ibc_ctx(&mut working_set)
            .store_host_consensus_state(Height::new(0, current_height).unwrap(), consensus_state)
            .unwrap();

        working_set.checkpoint()
    }
}
