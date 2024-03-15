//! Contains helper functions to manually setup the ibc module state
//! on the mock rollup.

use ibc_client_tendermint::types::client_type as tm_client_type;
use ibc_core::channel::types::channel::{
    ChannelEnd, Counterparty as ChanCounterparty, Order, State as ChannelState,
};
use ibc_core::channel::types::Version as ChannelVersion;
use ibc_core::client::context::client_state::ClientStateExecution;
use ibc_core::commitment_types::commitment::CommitmentPrefix;
use ibc_core::connection::types::version::Version as ConnectionVersion;
use ibc_core::connection::types::{
    ConnectionEnd, Counterparty as ConnCounterparty, State as ConnectionState,
};
use ibc_core::host::types::identifiers::{
    ChainId, ChannelId, ClientId, ConnectionId, PortId, Sequence,
};
use ibc_core::host::types::path::{
    ChannelEndPath, ConnectionPath, SeqAckPath, SeqRecvPath, SeqSendPath,
};
use ibc_core::host::{ExecutionContext, ValidationContext};
use sov_consensus_state_tracker::HasConsensusState;
use sov_ibc::context::IbcContext;
use sov_modules_api::{Spec, WorkingSet};
use sov_rollup_interface::services::da::DaService;
use sov_state::{MerkleProofSpec, ProverStorage};

use super::MockRollup;
use crate::cosmos::{dummy_tm_client_state, dummy_tm_consensus_state};

impl<S, Da, P> MockRollup<S, Da, P>
where
    S: Spec<Storage = ProverStorage<P>> + Send + Sync,
    Da: DaService<Error = anyhow::Error> + Clone,
    Da::Spec: HasConsensusState,
    P: MerkleProofSpec + Clone + 'static,
    <P as MerkleProofSpec>::Hasher: Send,
{
    /// Establishes a tendermint light client on the ibc module
    pub async fn setup_client(&mut self, client_chain_id: &ChainId) -> ClientId {
        let mut working_set = WorkingSet::new(self.prover_storage());

        let mut ibc_ctx: IbcContext<'_, S, <Da as DaService>::Spec> =
            self.ibc_ctx(&mut working_set);

        let client_counter = ibc_ctx.client_counter().unwrap();

        let client_id = tm_client_type().build_client_id(client_counter);

        let current_height = ibc_ctx.host_height().unwrap();

        let client_state = dummy_tm_client_state(client_chain_id.clone(), current_height);

        let consensus_state = dummy_tm_consensus_state();

        client_state
            .initialise(&mut ibc_ctx, &client_id, consensus_state.into())
            .unwrap();

        ibc_ctx.increase_client_counter().unwrap();

        self.commit(working_set.checkpoint().0).await;

        client_id
    }

    /// Establishes a connection on the ibc module with the `Open` state
    pub async fn setup_connection(
        &mut self,
        client_id: ClientId,
        prefix: CommitmentPrefix,
    ) -> ConnectionId {
        let mut working_set = WorkingSet::new(self.prover_storage());

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

        self.commit(working_set.checkpoint().0).await;

        connection_id
    }

    /// Establishes a channel on the ibc module with the `Open` state
    pub async fn setup_channel(&mut self, connection_id: ConnectionId) -> (PortId, ChannelId) {
        let mut working_set = WorkingSet::new(self.prover_storage());

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

        self.commit(working_set.checkpoint().0).await;

        (port_id, channel_id)
    }

    /// Sets the send sequence number for a given channel and port ids
    pub async fn with_send_sequence(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq_number: Sequence,
    ) {
        let mut working_set = WorkingSet::new(self.prover_storage());

        let mut ibc_ctx = self.ibc_ctx(&mut working_set);

        let seq_send_path = SeqSendPath::new(&port_id, &channel_id);

        ibc_ctx
            .store_next_sequence_send(&seq_send_path, seq_number)
            .unwrap();

        self.commit(working_set.checkpoint().0).await;
    }

    /// Sets the recv sequence number for a given channel and port ids
    pub async fn with_recv_sequence(
        &mut self,
        port_id: PortId,
        chan_id: ChannelId,
        seq_number: Sequence,
    ) {
        let mut working_set = WorkingSet::new(self.prover_storage());

        let mut ibc_ctx = self.ibc_ctx(&mut working_set);

        let seq_recv_path = SeqRecvPath::new(&port_id, &chan_id);

        ibc_ctx
            .store_next_sequence_recv(&seq_recv_path, seq_number)
            .unwrap();

        self.commit(working_set.checkpoint().0).await;
    }

    /// Sets the ack sequence number for a given channel and port ids
    pub async fn with_ack_sequence(
        &mut self,
        port_id: PortId,
        chan_id: ChannelId,
        seq_number: Sequence,
    ) {
        let mut working_set = WorkingSet::new(self.prover_storage());

        let mut ibc_ctx = self.ibc_ctx(&mut working_set);

        let seq_ack_path = SeqAckPath::new(&port_id, &chan_id);

        ibc_ctx
            .store_next_sequence_ack(&seq_ack_path, seq_number)
            .unwrap();

        self.commit(working_set.checkpoint().0).await;
    }
}
