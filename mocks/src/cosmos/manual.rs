//! Contains helper functions to manually setup the ibc module state
//! on the mock Cosmos chain.

use core::fmt::Debug;

use basecoin_app::modules::ibc::AnyConsensusState;
use basecoin_store::context::ProvableStore;
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
use ibc_core::commitment_types::commitment::CommitmentPrefix;
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
use tendermint::{Hash, Time};

use super::{dummy_tm_client_state, MockCosmosChain};

impl<S: ProvableStore + Default + Debug> MockCosmosChain<S> {
    /// Establishes a tendermint light client on the ibc module
    pub fn setup_client(&mut self, client_chain_id: &ChainId) -> ClientId {
        let client_counter = self.ibc_ctx().client_counter().unwrap();

        let client_id = tm_client_type().build_client_id(client_counter);

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
