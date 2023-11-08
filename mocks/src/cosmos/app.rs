use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use basecoin_app::modules::auth::Auth;
use basecoin_app::modules::bank::Bank;
use basecoin_app::modules::context::{prefix, Identifiable};
use basecoin_app::modules::ibc::{AnyConsensusState, Ibc, IbcContext};
use basecoin_app::modules::types::IdentifiedModule;
use basecoin_app::{BaseCoinApp, Builder};
use basecoin_store::context::{ProvableStore, Store};
use basecoin_store::impls::RevertibleStore;
use basecoin_store::utils::SharedRwExt;
use ibc::clients::ics07_tendermint::client_type as tm_client_type;
use ibc::clients::ics07_tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc::core::ics02_client::client_state::ClientStateCommon;
use ibc::core::ics02_client::ClientExecutionContext;
use ibc::core::ics03_connection::connection::{
    ConnectionEnd, Counterparty as ConnCounterparty, State as ConnectionState,
};
use ibc::core::ics03_connection::version::Version as ConnectionVersion;
use ibc::core::ics04_channel::channel::{
    ChannelEnd, Counterparty as ChanCounterparty, Order, State as ChannelState,
};
use ibc::core::ics04_channel::packet::Sequence;
use ibc::core::ics04_channel::Version as ChannelVersion;
use ibc::core::ics23_commitment::commitment::CommitmentProofBytes;
use ibc::core::ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::ics24_host::path::{
    ChannelEndPath, ClientConsensusStatePath, ClientStatePath, ConnectionPath, Path, SeqAckPath,
    SeqRecvPath, SeqSendPath,
};
use ibc::core::{ExecutionContext, ValidationContext};
use ibc::hosts::tendermint::IBC_QUERY_PATH;
use ibc::Height;
use tendermint::abci::request::{InitChain, Query};
use tendermint::block::Height as TmHeight;
use tendermint::v0_37::abci::{Request as AbciRequest, Response as AbciResponse};
use tendermint::{AppHash, Hash, Time};
use tendermint_testgen::consensus::default_consensus_params;
use tendermint_testgen::light_block::TmLightBlock;
use tendermint_testgen::{Generator, Header, LightBlock, Validator};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tower::Service;

use super::helpers::{
    convert_tm_to_ics_merkle_proof, dummy_tm_client_state, genesis_app_state, MutexUtil,
};

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

        let genesis_time = Time::now();

        let genesis_block = Self::generate_block(
            &chain_id,
            genesis_height.revision_height(),
            genesis_time,
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

        let new_tm_light_block = Self::generate_block(
            &self.chain_id,
            blocks.len() as u64 + 1,
            Time::now(),
            &validators,
            app_hash,
        );

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
    pub fn run(&self) -> JoinHandle<()> {
        let chain = self.clone();

        self.runtime().spawn(async move {
            chain.init().await;

            loop {
                chain.begin_block().await;

                tokio::time::sleep(Duration::from_millis(200)).await;

                chain.commit().await;
            }
        })
    }

    /// Queries the chain for a given path and height.
    pub async fn query(
        &self,
        path: impl Into<Path> + Send,
        height: &Height,
    ) -> (Vec<u8>, CommitmentProofBytes) {
        let request = Query {
            path: IBC_QUERY_PATH.to_string(),
            data: path.into().to_string().into_bytes().into(),
            height: TmHeight::try_from(height.revision_height()).unwrap(),
            prove: true,
        };

        let mut app = self.app.clone();

        let response = match app.call(AbciRequest::Query(request)).await.unwrap() {
            AbciResponse::Query(res) => res,
            _ => panic!("unexpected response from query"),
        };

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

        let client_state =
            dummy_tm_client_state(client_chain_id.clone(), Height::new(0, 3).unwrap());

        let latest_height = client_state.latest_height();

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
            .store_client_state(client_state_path, client_state)
            .unwrap();

        let consensus_state_path =
            ClientConsensusStatePath::new(&client_id, &Height::new(0, 3).unwrap());

        let consensus_state = AnyConsensusState::Tendermint(TmConsensusState::new(
            vec![].into(),
            Time::now(),
            Hash::None,
        ));

        self.ibc_ctx()
            .store_consensus_state(consensus_state_path, consensus_state)
            .unwrap();

        client_id
    }

    /// Establishes a connection on the ibc module with `Open` state
    pub fn setup_connection(&mut self, client_id: ClientId) -> ConnectionId {
        let connection_id = ConnectionId::new(0);

        let connection_path = ConnectionPath::new(&connection_id);

        let prefix = self.ibc_ctx().commitment_prefix();

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
