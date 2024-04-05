//! Defines JSON RPC methods exposed by the ibc module
use std::cell::RefCell;
use std::rc::Rc;

use borsh::BorshSerialize;
use ibc_core::channel::types::commitment::PacketCommitment;
use ibc_core::host::types::path::{ClientConsensusStatePath, CommitmentPath, UpgradeClientPath};
use ibc_core::host::ValidationContext;
use ibc_query::core::channel::{
    query_channel, query_channel_client_state, query_channel_consensus_state, query_channels,
    query_connection_channels, query_next_sequence_receive, query_packet_acknowledgement,
    query_packet_acknowledgements, query_packet_commitments, query_packet_receipt,
    query_unreceived_acks, query_unreceived_packets, QueryChannelClientStateRequest,
    QueryChannelClientStateResponse, QueryChannelConsensusStateRequest,
    QueryChannelConsensusStateResponse, QueryChannelRequest, QueryChannelResponse,
    QueryChannelsRequest, QueryChannelsResponse, QueryConnectionChannelsRequest,
    QueryConnectionChannelsResponse, QueryNextSequenceReceiveRequest,
    QueryNextSequenceReceiveResponse, QueryPacketAcknowledgementRequest,
    QueryPacketAcknowledgementResponse, QueryPacketAcknowledgementsRequest,
    QueryPacketAcknowledgementsResponse, QueryPacketCommitmentRequest,
    QueryPacketCommitmentResponse, QueryPacketCommitmentsRequest, QueryPacketCommitmentsResponse,
    QueryPacketReceiptRequest, QueryPacketReceiptResponse, QueryUnreceivedAcksRequest,
    QueryUnreceivedAcksResponse, QueryUnreceivedPacketsRequest, QueryUnreceivedPacketsResponse,
};
use ibc_query::core::client::{
    query_client_states, query_client_status, query_consensus_state_heights,
    query_consensus_states, QueryClientStateRequest, QueryClientStateResponse,
    QueryClientStatesRequest, QueryClientStatesResponse, QueryClientStatusRequest,
    QueryClientStatusResponse, QueryConsensusStateHeightsRequest,
    QueryConsensusStateHeightsResponse, QueryConsensusStateRequest, QueryConsensusStateResponse,
    QueryConsensusStatesRequest, QueryConsensusStatesResponse, QueryUpgradedClientStateRequest,
    QueryUpgradedClientStateResponse, QueryUpgradedConsensusStateRequest,
    QueryUpgradedConsensusStateResponse,
};
use ibc_query::core::connection::{
    query_client_connections, query_connection, query_connection_client_state,
    query_connection_consensus_state, query_connection_params, query_connections,
    QueryClientConnectionsRequest, QueryClientConnectionsResponse,
    QueryConnectionClientStateRequest, QueryConnectionClientStateResponse,
    QueryConnectionConsensusStateRequest, QueryConnectionConsensusStateResponse,
    QueryConnectionParamsRequest, QueryConnectionParamsResponse, QueryConnectionRequest,
    QueryConnectionResponse, QueryConnectionsRequest, QueryConnectionsResponse,
};
use jsonrpsee::core::RpcResult;
use sov_celestia_client::client_state::ClientState as HostClientState;
use sov_celestia_client::consensus_state::ConsensusState as HostConsensusState;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::{ProvenStateAccessor, Spec, WorkingSet};
use sov_state::storage::{SlotKey, StateCodec, StateItemCodec};

use crate::clients::{AnyClientState, AnyConsensusState};
use crate::context::IbcContext;
use crate::helpers::to_jsonrpsee_error;
use crate::Ibc;

/// Structure returned by the `client_state` rpc method.
#[rpc_gen(client, server, namespace = "ibc")]
impl<S: Spec> Ibc<S> {
    #[rpc_method(name = "clientState")]
    pub fn client_state(
        &self,
        request: QueryClientStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryClientStateResponse> {
        let proof_height = self.determine_query_height(request.query_height, working_set)?;

        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());

        let value_with_proof = self
            .client_state_map
            .get_with_proof(&request.client_id, &mut archival_working_set);

        let storage_value = value_with_proof.value.ok_or_else(|| {
            to_jsonrpsee_error(format!(
                "Client state not found for client {:?}",
                request.client_id
            ))
        })?;

        let client_state: AnyClientState = self
            .client_state_map
            .codec()
            .try_decode(storage_value.value())
            .map_err(to_jsonrpsee_error)?;

        let proof = value_with_proof
            .proof
            .try_to_vec()
            .map_err(to_jsonrpsee_error)?;

        Ok(QueryClientStateResponse::new(
            client_state.into(),
            proof,
            proof_height,
        ))
    }

    #[rpc_method(name = "clientStates")]
    pub fn client_states(
        &self,
        request: QueryClientStatesRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryClientStatesResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_client_states(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "consensusState")]
    pub fn consensus_state(
        &self,
        request: QueryConsensusStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConsensusStateResponse> {
        let prefix = self.consensus_state_map.prefix();

        let codec = self.consensus_state_map.codec();

        let consensus_height = match request.consensus_height {
            Some(height) => height,
            None => {
                return Err(to_jsonrpsee_error(
                    "Consensus height is required for querying consensus state",
                ))
            }
        };

        let path = ClientConsensusStatePath::new(
            request.client_id.clone(),
            consensus_height.revision_number(),
            consensus_height.revision_height(),
        );

        let key = SlotKey::new(prefix, &path, codec.key_codec());

        let value_with_proof = working_set.get_with_proof(key);

        let storage_value = value_with_proof.value.ok_or_else(|| {
            to_jsonrpsee_error(format!(
                "Consensus state not found for client {:?}",
                request.client_id
            ))
        })?;

        let consensus_state: AnyConsensusState = codec
            .try_decode(storage_value.value())
            .map_err(to_jsonrpsee_error)?;

        let proof = value_with_proof
            .proof
            .try_to_vec()
            .map_err(to_jsonrpsee_error)?;

        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        let proof_height = ibc_ctx.host_height().map_err(to_jsonrpsee_error)?;

        Ok(QueryConsensusStateResponse::new(
            consensus_state.into(),
            proof,
            proof_height,
        ))
    }

    #[rpc_method(name = "consensusStates")]
    pub fn consensus_states(
        &self,
        request: QueryConsensusStatesRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConsensusStatesResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_consensus_states(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "consensusStateHeights")]
    pub fn consensus_state_heights(
        &self,
        request: QueryConsensusStateHeightsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConsensusStateHeightsResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_consensus_state_heights(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "clientStatus")]
    pub fn client_status(
        &self,
        request: QueryClientStatusRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryClientStatusResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_client_status(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "upgradedClientState")]
    pub fn upgraded_client_state(
        &self,
        request: QueryUpgradedClientStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryUpgradedClientStateResponse> {
        let proof_height = self.determine_query_height(request.query_height, working_set)?;

        let upgrade_client_path =
            UpgradeClientPath::UpgradedClientState(proof_height.revision_height());

        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());

        let value_with_proof = self
            .upgraded_client_state_map
            .get_with_proof(&upgrade_client_path, &mut archival_working_set);

        let storage_value = value_with_proof.value.ok_or_else(|| {
            to_jsonrpsee_error(format!(
                "upgraded client state not found at: {proof_height:?}",
            ))
        })?;

        let upgraded_client_state: HostClientState = self
            .upgraded_client_state_map
            .codec()
            .try_decode(storage_value.value())
            .map_err(to_jsonrpsee_error)?;

        let proof = value_with_proof
            .proof
            .try_to_vec()
            .map_err(to_jsonrpsee_error)?;

        Ok(QueryUpgradedClientStateResponse::new(
            upgraded_client_state.into(),
            proof,
            proof_height,
        ))
    }

    #[rpc_method(name = "upgradedConsensusState")]
    pub fn upgraded_consensus_state(
        &self,
        request: QueryUpgradedConsensusStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryUpgradedConsensusStateResponse> {
        let proof_height = self.determine_query_height(request.query_height, working_set)?;

        let upgrade_consensus_path =
            UpgradeClientPath::UpgradedClientConsensusState(proof_height.revision_height());

        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());

        let value_with_proof = self
            .upgraded_consensus_state_map
            .get_with_proof(&upgrade_consensus_path, &mut archival_working_set);

        let storage_value = value_with_proof.value.ok_or_else(|| {
            to_jsonrpsee_error(format!(
                "upgraded consensus state not found at: {proof_height:?}",
            ))
        })?;

        let upgraded_consensus_state: HostConsensusState = self
            .upgraded_consensus_state_map
            .codec()
            .try_decode(storage_value.value())
            .map_err(to_jsonrpsee_error)?;

        let proof = value_with_proof
            .proof
            .try_to_vec()
            .map_err(to_jsonrpsee_error)?;

        Ok(QueryUpgradedConsensusStateResponse::new(
            upgraded_consensus_state.into(),
            proof,
            proof_height,
        ))
    }

    #[rpc_method(name = "connection")]
    pub fn connection(
        &self,
        request: QueryConnectionRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConnectionResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_connection(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "connections")]
    pub fn connections(
        &self,
        request: QueryConnectionsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConnectionsResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_connections(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "clientConnections")]
    pub fn client_connections(
        &self,
        request: QueryClientConnectionsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryClientConnectionsResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_client_connections(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "connectionClientState")]
    pub fn connection_client_state(
        &self,
        request: QueryConnectionClientStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConnectionClientStateResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_connection_client_state(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "connectionConsensusState")]
    pub fn connection_consensus_state(
        &self,
        request: QueryConnectionConsensusStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConnectionConsensusStateResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_connection_consensus_state(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "connectionParams")]
    pub fn connection_params(
        &self,
        request: QueryConnectionParamsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConnectionParamsResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_connection_params(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "channel")]
    pub fn channel(
        &self,
        request: QueryChannelRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryChannelResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_channel(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "channels")]
    pub fn channels(
        &self,
        request: QueryChannelsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryChannelsResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_channels(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "connectionChannels")]
    pub fn connection_channels(
        &self,
        request: QueryConnectionChannelsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConnectionChannelsResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_connection_channels(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "channelClientState")]
    pub fn channel_client_state(
        &self,
        request: QueryChannelClientStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryChannelClientStateResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_channel_client_state(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "channelConsensusState")]
    pub fn channel_consensus_state(
        &self,
        request: QueryChannelConsensusStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryChannelConsensusStateResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_channel_consensus_state(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "packetCommitment")]
    pub fn packet_commitment(
        &self,
        request: QueryPacketCommitmentRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryPacketCommitmentResponse> {
        let commitment_path =
            CommitmentPath::new(&request.port_id, &request.channel_id, request.sequence);

        let proof_height = self.determine_query_height(request.query_height, working_set)?;

        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());

        let value_with_proof = self
            .packet_commitment_map
            .get_with_proof(&commitment_path, &mut archival_working_set);

        let proof = value_with_proof
            .proof
            .try_to_vec()
            .map_err(to_jsonrpsee_error)?;

        let storage_value = value_with_proof.value.ok_or_else(|| {
            to_jsonrpsee_error(format!(
                "Packet commitment not found for path {commitment_path:?}"
            ))
        })?;

        Ok(QueryPacketCommitmentResponse::new(
            PacketCommitment::from(storage_value.value().to_vec()),
            proof,
            proof_height,
        ))
    }

    #[rpc_method(name = "packetCommitments")]
    pub fn packet_commitments(
        &self,
        request: QueryPacketCommitmentsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryPacketCommitmentsResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_packet_commitments(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "packetReceipt")]
    pub fn packet_receipt(
        &self,
        request: QueryPacketReceiptRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryPacketReceiptResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_packet_receipt(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "packetAcknowledgement")]
    pub fn packet_acknowledgement(
        &self,
        request: QueryPacketAcknowledgementRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryPacketAcknowledgementResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_packet_acknowledgement(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "packetAcknowledgements")]
    pub fn packet_acknowledgements(
        &self,
        request: QueryPacketAcknowledgementsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryPacketAcknowledgementsResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_packet_acknowledgements(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "unreceivedPackets")]
    pub fn unreceived_packets(
        &self,
        request: QueryUnreceivedPacketsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryUnreceivedPacketsResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_unreceived_packets(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "unreceivedAcks")]
    pub fn unreceived_acks(
        &self,
        request: QueryUnreceivedAcksRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryUnreceivedAcksResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_unreceived_acks(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "nextSequenceReceive")]
    pub fn next_sequence_receive(
        &self,
        request: QueryNextSequenceReceiveRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryNextSequenceReceiveResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_next_sequence_receive(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }
}
