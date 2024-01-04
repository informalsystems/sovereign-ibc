//! Contains helper functions to manually setup the ibc module state
//! on the mock rollup.

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
use sov_ibc::clients::{AnyClientState, AnyConsensusState};
use sov_ibc::context::IbcContext;
use sov_modules_api::{Context, WorkingSet};
use sov_rollup_interface::services::da::DaService;
use sov_state::{MerkleProofSpec, ProverStorage};
use tendermint::{Hash, Time};

use super::MockRollup;
use crate::cosmos::dummy_tm_client_state;

impl<C, Da, S> MockRollup<C, Da, S>
where
    C: Context<Storage = ProverStorage<S>> + Send + Sync,
    Da: DaService<Error = anyhow::Error> + Clone,
    S: MerkleProofSpec + Clone + 'static,
    <S as MerkleProofSpec>::Hasher: Send,
{
    /// Establishes a tendermint light client on the ibc module
    pub async fn setup_client(&mut self, client_chain_id: &ChainId) -> ClientId {
        let mut working_set = WorkingSet::new(self.prover_storage());

        let mut ibc_ctx: IbcContext<'_, C, <Da as DaService>::Spec> =
            self.ibc_ctx(&mut working_set);

        let client_counter = ibc_ctx.client_counter().unwrap();

        let client_id = tm_client_type().build_client_id(client_counter);

        let client_state_path = ClientStatePath::new(&client_id);

        let current_height = ibc_ctx.host_height().unwrap();

        let client_state = AnyClientState::Tendermint(
            dummy_tm_client_state(client_chain_id.clone(), current_height).into(),
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

        let current_height = ibc_ctx.host_height().unwrap();

        let consensus_state_path = ClientConsensusStatePath::new(
            client_id.clone(),
            current_height.revision_number(),
            current_height.revision_height(),
        );

        let consensus_state = AnyConsensusState::Tendermint(
            TmConsensusState::new(
                Vec::new().into(),
                Time::now(),
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

        self.commit(working_set.checkpoint()).await;

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
        let mut working_set = WorkingSet::new(self.prover_storage());

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
        let mut working_set = WorkingSet::new(self.prover_storage());

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
        let mut working_set = WorkingSet::new(self.prover_storage());

        let mut ibc_ctx = self.ibc_ctx(&mut working_set);

        let seq_ack_path = SeqAckPath::new(&port_id, &chan_id);

        ibc_ctx
            .store_next_sequence_ack(&seq_ack_path, seq_number)
            .unwrap();

        self.commit(working_set.checkpoint()).await;
    }
}
