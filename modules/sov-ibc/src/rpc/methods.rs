//! Defines JSON RPC methods exposed by the ibc module
use ibc_query::core::channel::{
    query_channels, query_connection_channels, query_packet_acknowledgements,
    query_packet_commitments, query_unreceived_acks, query_unreceived_packets,
    QueryChannelClientStateRequest, QueryChannelClientStateResponse,
    QueryChannelConsensusStateRequest, QueryChannelConsensusStateResponse, QueryChannelRequest,
    QueryChannelResponse, QueryChannelsRequest, QueryChannelsResponse,
    QueryConnectionChannelsRequest, QueryConnectionChannelsResponse,
    QueryNextSequenceReceiveRequest, QueryNextSequenceReceiveResponse,
    QueryPacketAcknowledgementRequest, QueryPacketAcknowledgementResponse,
    QueryPacketAcknowledgementsRequest, QueryPacketAcknowledgementsResponse,
    QueryPacketCommitmentRequest, QueryPacketCommitmentResponse, QueryPacketCommitmentsRequest,
    QueryPacketCommitmentsResponse, QueryPacketReceiptRequest, QueryPacketReceiptResponse,
    QueryUnreceivedAcksRequest, QueryUnreceivedAcksResponse, QueryUnreceivedPacketsRequest,
    QueryUnreceivedPacketsResponse,
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
    query_connection_params, query_connections, QueryClientConnectionsRequest,
    QueryClientConnectionsResponse, QueryConnectionClientStateRequest,
    QueryConnectionClientStateResponse, QueryConnectionConsensusStateRequest,
    QueryConnectionConsensusStateResponse, QueryConnectionParamsRequest,
    QueryConnectionParamsResponse, QueryConnectionRequest, QueryConnectionResponse,
    QueryConnectionsRequest, QueryConnectionsResponse,
};
use jsonrpsee::core::RpcResult;
use sov_ibc_transfer::to_jsonrpsee_error;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::{Spec, WorkingSet};

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
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.client_state_response(request)
        })
    }

    #[rpc_method(name = "clientStates")]
    pub fn client_states(
        &self,
        request: QueryClientStatesRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryClientStatesResponse> {
        self.handle_request(None, working_set, |ibc_ctx| {
            query_client_states(ibc_ctx, &request).map_err(to_jsonrpsee_error)
        })
    }

    #[rpc_method(name = "consensusState")]
    pub fn consensus_state(
        &self,
        request: QueryConsensusStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConsensusStateResponse> {
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.consensus_state_response(request)
        })
    }

    #[rpc_method(name = "consensusStates")]
    pub fn consensus_states(
        &self,
        request: QueryConsensusStatesRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConsensusStatesResponse> {
        self.handle_request(None, working_set, |ibc_ctx| {
            query_consensus_states(ibc_ctx, &request).map_err(to_jsonrpsee_error)
        })
    }

    #[rpc_method(name = "consensusStateHeights")]
    pub fn consensus_state_heights(
        &self,
        request: QueryConsensusStateHeightsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConsensusStateHeightsResponse> {
        self.handle_request(None, working_set, |ibc_ctx| {
            query_consensus_state_heights(ibc_ctx, &request).map_err(to_jsonrpsee_error)
        })
    }

    #[rpc_method(name = "clientStatus")]
    pub fn client_status(
        &self,
        request: QueryClientStatusRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryClientStatusResponse> {
        self.handle_request(None, working_set, |ibc_ctx| {
            query_client_status(ibc_ctx, &request).map_err(to_jsonrpsee_error)
        })
    }

    #[rpc_method(name = "upgradedClientState")]
    pub fn upgraded_client_state(
        &self,
        request: QueryUpgradedClientStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryUpgradedClientStateResponse> {
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.upgraded_client_state_response(request)
        })
    }

    #[rpc_method(name = "upgradedConsensusState")]
    pub fn upgraded_consensus_state(
        &self,
        request: QueryUpgradedConsensusStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryUpgradedConsensusStateResponse> {
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.upgraded_consensus_state_response(request)
        })
    }

    #[rpc_method(name = "connection")]
    pub fn connection(
        &self,
        request: QueryConnectionRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConnectionResponse> {
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.connection_response(request)
        })
    }

    #[rpc_method(name = "connections")]
    pub fn connections(
        &self,
        request: QueryConnectionsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConnectionsResponse> {
        self.handle_request(None, working_set, |ibc_ctx| {
            query_connections(ibc_ctx, &request).map_err(to_jsonrpsee_error)
        })
    }

    #[rpc_method(name = "clientConnections")]
    pub fn client_connections(
        &self,
        request: QueryClientConnectionsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryClientConnectionsResponse> {
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.client_connections_response(request)
        })
    }

    #[rpc_method(name = "connectionClientState")]
    pub fn connection_client_state(
        &self,
        request: QueryConnectionClientStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConnectionClientStateResponse> {
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.connection_client_state_response(request)
        })
    }

    #[rpc_method(name = "connectionConsensusState")]
    pub fn connection_consensus_state(
        &self,
        request: QueryConnectionConsensusStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConnectionConsensusStateResponse> {
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.connection_consensus_state_response(request)
        })
    }

    #[rpc_method(name = "connectionParams")]
    pub fn connection_params(
        &self,
        request: QueryConnectionParamsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConnectionParamsResponse> {
        self.handle_request(None, working_set, |ibc_ctx| {
            query_connection_params(ibc_ctx, &request).map_err(to_jsonrpsee_error)
        })
    }

    #[rpc_method(name = "channel")]
    pub fn channel(
        &self,
        request: QueryChannelRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryChannelResponse> {
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.channel_response(request)
        })
    }

    #[rpc_method(name = "channels")]
    pub fn channels(
        &self,
        request: QueryChannelsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryChannelsResponse> {
        self.handle_request(None, working_set, |ibc_ctx| {
            query_channels(ibc_ctx, &request).map_err(to_jsonrpsee_error)
        })
    }

    #[rpc_method(name = "connectionChannels")]
    pub fn connection_channels(
        &self,
        request: QueryConnectionChannelsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConnectionChannelsResponse> {
        self.handle_request(None, working_set, |ibc_ctx| {
            query_connection_channels(ibc_ctx, &request).map_err(to_jsonrpsee_error)
        })
    }

    #[rpc_method(name = "channelClientState")]
    pub fn channel_client_state(
        &self,
        request: QueryChannelClientStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryChannelClientStateResponse> {
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.channel_client_state_response(request)
        })
    }

    #[rpc_method(name = "channelConsensusState")]
    pub fn channel_consensus_state(
        &self,
        request: QueryChannelConsensusStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryChannelConsensusStateResponse> {
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.channel_consensus_state_response(request)
        })
    }

    #[rpc_method(name = "packetCommitment")]
    pub fn packet_commitment(
        &self,
        request: QueryPacketCommitmentRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryPacketCommitmentResponse> {
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.packet_commitment_response(request)
        })
    }

    #[rpc_method(name = "packetCommitments")]
    pub fn packet_commitments(
        &self,
        request: QueryPacketCommitmentsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryPacketCommitmentsResponse> {
        self.handle_request(None, working_set, |ibc_ctx| {
            query_packet_commitments(ibc_ctx, &request).map_err(to_jsonrpsee_error)
        })
    }

    #[rpc_method(name = "packetReceipt")]
    pub fn packet_receipt(
        &self,
        request: QueryPacketReceiptRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryPacketReceiptResponse> {
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.packet_receipt_response(request)
        })
    }

    #[rpc_method(name = "packetAcknowledgement")]
    pub fn packet_acknowledgement(
        &self,
        request: QueryPacketAcknowledgementRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryPacketAcknowledgementResponse> {
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.packet_acknowledgement_response(request)
        })
    }

    #[rpc_method(name = "packetAcknowledgements")]
    pub fn packet_acknowledgements(
        &self,
        request: QueryPacketAcknowledgementsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryPacketAcknowledgementsResponse> {
        self.handle_request(None, working_set, |ibc_ctx| {
            query_packet_acknowledgements(ibc_ctx, &request).map_err(to_jsonrpsee_error)
        })
    }

    #[rpc_method(name = "unreceivedPackets")]
    pub fn unreceived_packets(
        &self,
        request: QueryUnreceivedPacketsRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryUnreceivedPacketsResponse> {
        self.handle_request(None, working_set, |ibc_ctx| {
            query_unreceived_packets(ibc_ctx, &request).map_err(to_jsonrpsee_error)
        })
    }

    #[rpc_method(name = "unreceivedAcks")]
    pub fn unreceived_acks(
        &self,
        request: QueryUnreceivedAcksRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryUnreceivedAcksResponse> {
        self.handle_request(None, working_set, |ibc_ctx| {
            query_unreceived_acks(ibc_ctx, &request).map_err(to_jsonrpsee_error)
        })
    }

    #[rpc_method(name = "nextSequenceReceive")]
    pub fn next_sequence_receive(
        &self,
        request: QueryNextSequenceReceiveRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryNextSequenceReceiveResponse> {
        self.handle_request(request.query_height, working_set, |ibc_ctx| {
            ibc_ctx.next_sequence_receive_response(request)
        })
    }
}
