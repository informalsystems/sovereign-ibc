use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ibc_client_tendermint::types::{
    client_type as tm_client_type, ConsensusState as TmConsensusState,
};
use ibc_core::channel::types::channel::{
    ChannelEnd, Counterparty as ChanCounterparty, Order, State as ChannelState,
};
use ibc_core::channel::types::Version as ChannelVersion;
use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::client::context::ClientExecutionContext;
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::{CommitmentPrefix, CommitmentRoot};
use ibc_core::connection::types::version::Version as ConnectionVersion;
use ibc_core::connection::types::{
    ConnectionEnd, Counterparty as ConnCounterparty, State as ConnectionState,
};
use ibc_core::handler::types::events::IbcEvent;
use ibc_core::host::types::identifiers::{
    ChainId, ChannelId, ClientId, ConnectionId, PortId, Sequence,
};
use ibc_core::host::types::path::{
    ChannelEndPath, ClientConsensusStatePath, ClientStatePath, ConnectionPath, SeqAckPath,
    SeqRecvPath, SeqSendPath,
};
use ibc_core::host::{ExecutionContext, ValidationContext};
use sov_bank::TokenConfig;
use sov_ibc::call::CallMessage;
use sov_ibc::clients::{AnyClientState, AnyConsensusState};
use sov_ibc::context::IbcContext;
use sov_modules_api::hooks::{FinalizeHook, SlotHooks};
use sov_modules_api::{Context, DispatchCall, Genesis, SlotData, StateCheckpoint, WorkingSet};
use sov_rollup_interface::services::da::DaService;
use sov_state::{MerkleProofSpec, ProverStorage, Storage};
use tendermint::{Hash, Time};

use super::config::TestConfig;
use super::runtime::{GenesisConfig, Runtime};
use crate::cosmos::helpers::dummy_tm_client_state;
use crate::sovereign::runtime::RuntimeCall;
use crate::JAN_1_2023;
/// Defines a mock rollup structure with default configurations and specs
#[derive(Clone)]
pub struct MockRollup<C, Da, S>
where
    C: Context,
    Da: DaService<Error = anyhow::Error> + Clone,
    S: MerkleProofSpec,
{
    chain_id: ChainId,
    config: TestConfig<C>,
    runtime: Runtime<C, Da::Spec>,
    prover_storage: ProverStorage<S>,
    da_service: Da,
    rollup_ctx: C,
    state_root: <ProverStorage<S> as Storage>::Root,
}

impl<C, Da, S> MockRollup<C, Da, S>
where
    C: Context<Storage = ProverStorage<S>> + Send + Sync,
    Da: DaService<Error = anyhow::Error> + Clone,
    <Da as DaService>::Spec: Clone,
    S: MerkleProofSpec + Clone + 'static,
    <S as MerkleProofSpec>::Hasher: Send,
{
    pub fn new(
        chain_id: ChainId,
        config: TestConfig<C>,
        runtime: Runtime<C, Da::Spec>,
        prover_storage: ProverStorage<S>,
        rollup_ctx: C,
        da_service: Da,
    ) -> Self {
        Self {
            chain_id,
            config,
            runtime,
            prover_storage,
            da_service,
            rollup_ctx,
            state_root: jmt::RootHash([0; 32]),
        }
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    pub fn rollup_ctx(&self) -> C {
        self.rollup_ctx.clone()
    }

    pub fn config(&self) -> &TestConfig<C> {
        &self.config
    }

    pub fn runtime(&self) -> &Runtime<C, Da::Spec> {
        &self.runtime
    }

    pub fn prover_storage(&self) -> ProverStorage<S> {
        self.prover_storage.clone()
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

    /// Returns the balance of a user for a given token
    pub fn get_balance_of(&self, user_address: C::Address, token_address: C::Address) -> u64 {
        let mut working_set = WorkingSet::new(self.prover_storage.clone());

        self.runtime()
            .bank
            .get_balance_of(user_address, token_address, &mut working_set)
            .unwrap()
    }

    /// Returns token address of an IBC denom
    pub fn get_minted_token_address(&self, token_denom: String) -> Option<C::Address> {
        let mut working_set = WorkingSet::new(self.prover_storage.clone());

        self.runtime
            .ibc_transfer
            .minted_token(token_denom, &mut working_set)
            .map(|token| token.address)
            .ok()
    }

    /// Searches the transfer module to retrieve the address of the token held
    /// in escrow, based on its token denom.
    pub fn get_escrowed_token_address(&self, token_denom: String) -> Option<C::Address> {
        let mut working_set = WorkingSet::new(self.prover_storage.clone());

        self.runtime()
            .ibc_transfer
            .escrowed_token(token_denom, &mut working_set)
            .map(|token| token.address)
            .ok()
    }

    fn set_state_root(&mut self, state_root: <ProverStorage<S> as Storage>::Root) {
        self.state_root = state_root;
    }

    /// Sets the host consensus state when processing each block
    fn set_host_consensus_state(
        &mut self,
        checkpoint: StateCheckpoint<C>,
        root_hash: <ProverStorage<S> as Storage>::Root,
    ) -> StateCheckpoint<C> {
        let mut working_set = checkpoint.to_revertable();

        let current_height = self.runtime().chain_state.get_slot_height(&mut working_set);

        let consensus_state = AnyConsensusState::Tendermint(
            TmConsensusState::new(
                CommitmentRoot::from_bytes(&root_hash.0),
                Time::from_unix_timestamp(JAN_1_2023 + current_height as i64, 0).unwrap(),
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

    /// Initializes the chain with the genesis configuration
    pub async fn init_chain(&mut self) -> StateCheckpoint<C> {
        let mut working_set = WorkingSet::new(self.prover_storage.clone());

        let genesis_config = GenesisConfig::new(
            self.config.chain_state_config.clone(),
            self.config.bank_config.clone(),
            self.config.ibc_config.clone(),
            self.config.ibc_transfer_config.clone(),
        );

        self.runtime
            .genesis(&genesis_config, &mut working_set)
            .unwrap();

        self.commit(working_set.checkpoint()).await
    }

    /// Begins a block by setting the host consensus state and triggering the slot hook
    pub async fn begin_block(&mut self, checkpoint: StateCheckpoint<C>) -> StateCheckpoint<C> {
        let mut working_set = checkpoint.to_revertable();

        let current_height = self.runtime().chain_state.get_slot_height(&mut working_set);

        // Dummy transaction to trigger the block generation
        self.da_service.send_transaction(&[0; 32]).await.unwrap();

        let block = self
            .da_service
            .get_block_at(current_height - 1)
            .await
            .unwrap();

        self.runtime.begin_slot_hook(
            block.header(),
            &block.validity_condition(),
            &self.state_root,
            &mut working_set,
        );

        self.set_host_consensus_state(working_set.checkpoint(), self.state_root)
    }

    /// Commits a block by triggering the end slot hook, computing the state
    /// update and committing it to the prover storage
    pub async fn commit(&mut self, checkpoint: StateCheckpoint<C>) -> StateCheckpoint<C> {
        let checkpoint = self.begin_block(checkpoint).await;

        let mut working_set = checkpoint.to_revertable();

        self.runtime.end_slot_hook(&mut working_set);

        let mut checkpoint = working_set.checkpoint();

        let (cache_log, witness) = checkpoint.freeze();

        let (root_hash, state_update) = self
            .prover_storage
            .compute_state_update(cache_log, &witness)
            .expect("jellyfish merkle tree update must succeed");

        let mut working_set = checkpoint.to_revertable();

        self.runtime
            .finalize_hook(&root_hash, &mut working_set.accessory_state());

        let mut checkpoint = working_set.checkpoint();

        let accessory_log = checkpoint.freeze_non_provable();

        self.prover_storage.commit(&state_update, &accessory_log);

        self.set_state_root(root_hash);

        checkpoint
    }

    /// Runs the rollup chain by initializing the chain and then committing
    /// blocks at a fixed interval
    pub async fn run(&mut self) {
        self.init_chain().await;

        let mut chain = self.clone();

        tokio::task::spawn(async move {
            loop {
                let working_set = WorkingSet::new(chain.prover_storage.clone());

                tokio::time::sleep(Duration::from_millis(200)).await;

                chain.commit(working_set.checkpoint()).await;
            }
        });
    }

    /// Applies an IBC message to the execution layer
    pub async fn apply_msg(&mut self, msg: Vec<CallMessage>) -> Vec<IbcEvent> {
        let mut working_set = WorkingSet::new(self.prover_storage());

        for m in msg {
            self.runtime()
                .dispatch_call(RuntimeCall::ibc(m), &mut working_set, &self.rollup_ctx())
                .unwrap();
        }

        let _ = working_set.events();

        self.commit(working_set.checkpoint()).await;

        vec![]
    }

    /// Establishes a tendermint light client on the ibc module
    pub async fn setup_client(&mut self, client_chain_id: &ChainId) -> ClientId {
        let mut working_set = WorkingSet::new(self.prover_storage.clone());

        let mut ibc_ctx = self.ibc_ctx(&mut working_set);

        let client_counter = ibc_ctx.client_counter().unwrap();

        let client_id = ClientId::new(tm_client_type(), client_counter).unwrap();

        let client_state_path = ClientStatePath::new(&client_id);

        let client_state = AnyClientState::Tendermint(
            dummy_tm_client_state(client_chain_id.clone(), Height::new(0, 3).unwrap()).into(),
        );

        let latest_height = client_state.latest_height();

        ibc_ctx
            .store_update_time(
                client_id.clone(),
                latest_height,
                ibc_ctx.host_timestamp().unwrap(),
            )
            .unwrap();

        ibc_ctx
            .store_update_height(
                client_id.clone(),
                latest_height,
                ibc_ctx.host_height().unwrap(),
            )
            .unwrap();

        ibc_ctx.increase_client_counter().unwrap();

        ibc_ctx
            .store_client_state(client_state_path, client_state)
            .unwrap();

        let consensus_state_path = ClientConsensusStatePath::new(client_id.clone(), 0, 3);

        let consensus_state = AnyConsensusState::Tendermint(
            TmConsensusState::new(
                vec![].into(),
                Time::from_unix_timestamp(JAN_1_2023, 0).unwrap(),
                // Hash for default validator set of CosmosBuilder
                Hash::Sha256([
                    0xd6, 0xb9, 0x39, 0x22, 0xc3, 0x3a, 0xae, 0xbe, 0xc9, 0x4, 0x35, 0x66, 0xcb,
                    0x4b, 0x1b, 0x48, 0x36, 0x5b, 0x13, 0x58, 0xb6, 0x7c, 0x7d, 0xef, 0x98, 0x6d,
                    0x9e, 0xe1, 0x86, 0x1b, 0xc1, 0x43,
                ]),
            )
            .into(),
        );

        ibc_ctx
            .store_consensus_state(consensus_state_path, consensus_state)
            .unwrap();

        self.commit(working_set.checkpoint()).await;

        client_id
    }

    /// Establishes a connection on the ibc module with the `Open` state
    pub async fn setup_connection(
        &mut self,
        client_id: ClientId,
        prefix: CommitmentPrefix,
    ) -> ConnectionId {
        let mut working_set = WorkingSet::new(self.prover_storage.clone());

        let mut ibc_ctx = self.ibc_ctx(&mut working_set);

        let connection_id = ConnectionId::new(0);

        let connection_path = ConnectionPath::new(&connection_id);

        let connection_end = ConnectionEnd::new(
            ConnectionState::Open,
            client_id.clone(),
            ConnCounterparty::new(client_id, Some(connection_id.clone()), prefix),
            vec![ConnectionVersion::default()],
            Default::default(),
        )
        .unwrap();

        ibc_ctx
            .store_connection(&connection_path, connection_end)
            .unwrap();

        self.commit(working_set.checkpoint()).await;

        connection_id
    }

    /// Establishes a channel on the ibc module with the `Open` state
    pub async fn setup_channel(&mut self, connection_id: ConnectionId) -> (PortId, ChannelId) {
        let mut working_set = WorkingSet::new(self.prover_storage.clone());

        let mut ibc_ctx = self.ibc_ctx(&mut working_set);

        let channel_id = ChannelId::new(0);

        let port_id = PortId::transfer();

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let channel_end = ChannelEnd::new(
            ChannelState::Open,
            Order::default(),
            ChanCounterparty::new(PortId::transfer(), Some(channel_id.clone())),
            vec![connection_id],
            ChannelVersion::default(),
        )
        .unwrap();

        ibc_ctx
            .store_channel(&channel_end_path, channel_end)
            .unwrap();

        self.commit(working_set.checkpoint()).await;

        (port_id, channel_id)
    }

    /// Sets the send sequence number for a given channel and port ids
    pub async fn with_send_sequence(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq_number: Sequence,
    ) {
        let mut working_set = WorkingSet::new(self.prover_storage.clone());

        let mut ibc_ctx = self.ibc_ctx(&mut working_set);

        let seq_send_path = SeqSendPath::new(&port_id, &channel_id);

        ibc_ctx
            .store_next_sequence_send(&seq_send_path, seq_number)
            .unwrap();

        self.commit(working_set.checkpoint()).await;
    }

    /// Sets the recv sequence number for a given channel and port ids
    pub async fn with_recv_sequence(
        &mut self,
        port_id: PortId,
        chan_id: ChannelId,
        seq_number: Sequence,
    ) {
        let mut working_set = WorkingSet::new(self.prover_storage.clone());

        let mut ibc_ctx = self.ibc_ctx(&mut working_set);

        let seq_recv_path = SeqRecvPath::new(&port_id, &chan_id);

        ibc_ctx
            .store_next_sequence_recv(&seq_recv_path, seq_number)
            .unwrap();

        self.commit(working_set.checkpoint()).await;
    }

    /// Sets the ack sequence number for a given channel and port ids
    pub async fn with_ack_sequence(
        &mut self,
        port_id: PortId,
        chan_id: ChannelId,
        seq_number: Sequence,
    ) {
        let mut working_set = WorkingSet::new(self.prover_storage.clone());

        let mut ibc_ctx = self.ibc_ctx(&mut working_set);

        let seq_ack_path = SeqAckPath::new(&port_id, &chan_id);

        ibc_ctx
            .store_next_sequence_ack(&seq_ack_path, seq_number)
            .unwrap();

        self.commit(working_set.checkpoint()).await;
    }
}
