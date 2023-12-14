use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use basecoin_app::abci::v0_37::impls::query as basecoin_query;
use basecoin_app::modules::auth::proto::AccountId;
use basecoin_app::modules::auth::Auth;
use basecoin_app::modules::bank::{Bank, BankReader, Denom};
use basecoin_app::modules::context::{prefix, Identifiable};
use basecoin_app::modules::ibc::{AnyConsensusState, Ibc, IbcContext};
use basecoin_app::modules::types::IdentifiedModule;
use basecoin_app::{BaseCoinApp, Builder};
use basecoin_store::context::{ProvableStore, Store};
use basecoin_store::impls::RevertibleStore;
use basecoin_store::utils::SharedRwExt;
use ibc_client_tendermint::client_state::ClientState as TmClientState;
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
use ibc_core::commitment_types::commitment::{CommitmentPrefix, CommitmentProofBytes};
use ibc_core::connection::types::version::Version as ConnectionVersion;
use ibc_core::connection::types::{
    ConnectionEnd, Counterparty as ConnCounterparty, State as ConnectionState,
};
use ibc_core::host::types::identifiers::{
    ChainId, ChannelId, ClientId, ConnectionId, PortId, Sequence,
};
use ibc_core::host::types::path::{
    ChannelEndPath, ClientConsensusStatePath, ClientStatePath, ConnectionPath, SeqAckPath,
    SeqRecvPath, SeqSendPath,
};
use ibc_core::host::{ExecutionContext, ValidationContext};
use tendermint::abci::request::{InitChain, Query as RequestQuery};
use tendermint::abci::response::Query as ResponseQuery;
use tendermint::block::Height as TmHeight;
use tendermint::v0_37::abci::Request as AbciRequest;
use tendermint::{AppHash, Hash, Time};
use tendermint_testgen::consensus::default_consensus_params;
use tendermint_testgen::light_block::TmLightBlock;
use tendermint_testgen::{Generator, Header, LightBlock, Validator};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tower::Service;
use tracing::debug;

use super::helpers::{
    convert_tm_to_ics_merkle_proof, dummy_tm_client_state, genesis_app_state, MutexUtil,
};
use crate::utils::wait_for_block;

/// Defines a mock Cosmos chain that includes simplified store, application,
/// consensus layers.
#[derive(Clone)]
pub struct MockCosmosChain<S>
where
    S: ProvableStore + Debug,
{
    pub runtime: Arc<Runtime>,
    /// Chain identifier
    pub chain_id: ChainId,
    /// Chain validators
    pub validators: Arc<Mutex<Vec<Validator>>>,
    /// Chain blocks
    pub blocks: Arc<Mutex<Vec<TmLightBlock>>>,
    /// Chain application
    pub app: BaseCoinApp<S>,
}

impl<S: ProvableStore + Default + Debug> MockCosmosChain<S> {
    /// Constructs a new mock cosmos chain instance.
    pub fn new(
        runtime: Arc<Runtime>,
        chain_id: ChainId,
        validators: Vec<Validator>,
        store: S,
    ) -> Self {
        let app_builder = Builder::new(store);

        let auth = Auth::new(app_builder.module_store(&prefix::Auth {}.identifier()));
        let bank = Bank::new(
            app_builder.module_store(&prefix::Bank {}.identifier()),
            auth.account_reader().clone(),
            auth.account_keeper().clone(),
        );

        let ibc = Ibc::new(
            app_builder.module_store(&prefix::Ibc {}.identifier()),
            bank.bank_keeper().clone(),
        );

        // register modules with the app
        let app = app_builder
            .add_module(prefix::Auth {}.identifier(), auth)
            .add_module(prefix::Bank {}.identifier(), bank)
            .add_module(prefix::Ibc {}.identifier(), ibc)
            .build();

        let genesis_height = Height::new(chain_id.revision_number(), 1).expect("never fails");

        let genesis_block = Self::generate_block(
            &chain_id,
            genesis_height.revision_height(),
            Time::now(),
            &validators,
            AppHash::default(),
        );

        Self {
            runtime,
            chain_id,
            validators: Arc::new(Mutex::new(validators)),
            blocks: Arc::new(Mutex::new(vec![genesis_block])),
            app,
        }
    }

    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    pub fn ibc_ctx(&self) -> IbcContext<RevertibleStore<S>> {
        self.app.ibc().ctx()
    }

    pub fn get_balance_of(&self, denom: &str, account: String) -> Option<u64> {
        let account_id: AccountId = account.parse().unwrap();

        let denom = Denom(denom.to_string());

        if let Some(coin) = self
            .app
            .bank()
            .balance_reader()
            .get_all_balances(account_id)
            .into_iter()
            .find(|c| c.denom == denom)
        {
            Some(coin.amount.try_into().ok()?)
        } else {
            None
        }
    }

    pub fn get_blocks(&self) -> Vec<TmLightBlock> {
        self.blocks.acquire_mutex().clone()
    }

    /// Generates a new light block for the chain with the given parameters.
    pub fn generate_block(
        chain_id: &ChainId,
        height: u64,
        time: Time,
        validators: &[Validator],
        app_hash: AppHash,
    ) -> TmLightBlock {
        let header = Header::new(validators)
            .chain_id(&chain_id.to_string())
            .height(height)
            .time(time)
            .next_validators(validators)
            .app_hash(app_hash);

        LightBlock::new_default_with_header(header)
            .generate()
            .expect("failed to generate light block")
    }

    /// Grows the chain by one block.
    pub fn grow_blocks(&self) {
        let root_hash = self.app.store.root_hash();

        let app_hash = AppHash::try_from(root_hash).expect("invalid app hash");

        let mut blocks = self.blocks.acquire_mutex();

        let validators = self.validators.acquire_mutex();

        let height = blocks.len() as u64 + 1;

        debug!("cosmos: growing chain to height {}", height);

        let new_tm_light_block =
            Self::generate_block(&self.chain_id, height, Time::now(), &validators, app_hash);

        blocks.push(new_tm_light_block);
    }

    /// Initialize the chain with the app state.
    async fn init(&self) {
        let app_state = serde_json::to_vec(&genesis_app_state()).expect("infallible serialization");

        let request = InitChain {
            time: Time::now(),
            chain_id: self.chain_id.to_string(),
            consensus_params: default_consensus_params(),
            validators: vec![],
            app_state_bytes: app_state.into(),
            initial_height: TmHeight::from(1_u8),
        };

        let mut app = self.app.clone();

        app.call(AbciRequest::InitChain(request))
            .await
            .expect("failed to initialize chain");
    }

    /// Begins a new block on the chain.
    async fn begin_block(&self) {
        self.grow_blocks();

        let last_block = self.blocks.acquire_mutex().last().unwrap().clone();

        let mut events = Vec::new();

        let mut modules = self.app.modules.write_access();

        for IdentifiedModule { id: _, module } in modules.iter_mut() {
            let event = module.begin_block(&last_block.signed_header.header);
            events.extend(event);
        }
    }

    /// Commits the chain state to the store.
    async fn commit(&self) {
        let mut modules = self.app.modules.write_access();

        let mut state = self.app.store.write_access();

        for IdentifiedModule { id, module } in modules.iter_mut() {
            module
                .store_mut()
                .commit()
                .expect("failed to commit to state");

            state
                .set(id.clone().into(), module.store().root_hash())
                .expect("failed to update sub-store commitment");
        }

        state.commit().expect("failed to commit to state");
    }

    /// Runs the chain in a separate thread.
    pub async fn run(&self) -> JoinHandle<()> {
        let chain = self.clone();

        let handle = self.runtime().spawn(async move {
            chain.init().await;

            loop {
                chain.begin_block().await;

                tokio::time::sleep(Duration::from_millis(200)).await;

                chain.commit().await;
            }
        });

        wait_for_block().await;

        handle
    }

    /// Queries the chain for a given path and height.
    pub fn query(
        &self,
        data: Vec<u8>,
        path: String,
        height: &Height,
    ) -> (Vec<u8>, CommitmentProofBytes) {
        let response: ResponseQuery = basecoin_query(
            &self.app,
            RequestQuery {
                data: data.into(),
                path,
                height: height.revision_height().try_into().unwrap(),
                prove: true,
            }
            .into(),
        )
        .try_into()
        .unwrap();

        let proof = match response.proof {
            Some(proof) => proof,
            None => panic!("proof not found in query response"),
        };

        let merkle_proof = convert_tm_to_ics_merkle_proof(&proof);

        let commitment_proof = merkle_proof.try_into().unwrap();

        (response.value.into(), commitment_proof)
    }

    /// Establishes a tendermint light client on the ibc module
    pub fn setup_client(&mut self, client_chain_id: &ChainId) -> ClientId {
        let client_counter = self.ibc_ctx().client_counter().unwrap();

        let client_id = ClientId::new(tm_client_type(), client_counter).unwrap();

        let client_state_path = ClientStatePath::new(&client_id);

        let current_height = self.ibc_ctx().host_height().unwrap();

        let client_state = dummy_tm_client_state(client_chain_id.clone(), current_height);

        let latest_height = TmClientState::from(client_state.clone()).latest_height();

        self.ibc_ctx()
            .store_update_time(
                client_id.clone(),
                latest_height,
                self.ibc_ctx().host_timestamp().unwrap(),
            )
            .unwrap();

        self.ibc_ctx()
            .store_update_height(
                client_id.clone(),
                latest_height,
                self.ibc_ctx().host_height().unwrap(),
            )
            .unwrap();

        self.ibc_ctx().increase_client_counter().unwrap();

        self.ibc_ctx()
            .store_client_state(client_state_path, client_state.into())
            .unwrap();

        let consensus_state_path = ClientConsensusStatePath::new(
            client_id.clone(),
            current_height.revision_number(),
            current_height.revision_height(),
        );

        let consensus_state = AnyConsensusState::Tendermint(
            TmConsensusState::new(vec![].into(), Time::now(), Hash::None).into(),
        );

        self.ibc_ctx()
            .store_consensus_state(consensus_state_path, consensus_state)
            .unwrap();

        client_id
    }

    /// Establishes a connection on the ibc module with `Open` state
    pub fn setup_connection(
        &mut self,
        client_id: ClientId,
        prefix: CommitmentPrefix,
    ) -> ConnectionId {
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

        self.ibc_ctx()
            .store_connection(&connection_path, connection_end)
            .unwrap();

        connection_id
    }

    /// Establishes a channel on the ibc module with `Open` state
    pub fn setup_channel(&mut self, connection_id: ConnectionId) -> (PortId, ChannelId) {
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

        self.ibc_ctx()
            .store_channel(&channel_end_path, channel_end)
            .unwrap();

        (port_id, channel_id)
    }

    /// Sets the send sequence number for a given channel and port ids
    pub fn with_send_sequence(&self, port_id: PortId, channel_id: ChannelId, seq_number: Sequence) {
        let seq_send_path = SeqSendPath::new(&port_id, &channel_id);

        self.ibc_ctx()
            .store_next_sequence_send(&seq_send_path, seq_number)
            .unwrap();
    }

    /// Sets the receive sequence number for a given channel and port ids
    pub fn with_recv_sequence(&self, port_id: PortId, chan_id: ChannelId, seq_number: Sequence) {
        let seq_recv_path = SeqRecvPath::new(&port_id, &chan_id);

        self.ibc_ctx()
            .store_next_sequence_recv(&seq_recv_path, seq_number)
            .unwrap();
    }

    /// Sets the ack sequence number for a given channel and port ids
    pub fn with_ack_sequence(&self, port_id: PortId, chan_id: ChannelId, seq_number: Sequence) {
        let seq_ack_path = SeqAckPath::new(&port_id, &chan_id);

        self.ibc_ctx()
            .store_next_sequence_ack(&seq_ack_path, seq_number)
            .unwrap();
    }
}
