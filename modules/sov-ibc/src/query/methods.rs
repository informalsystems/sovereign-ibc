//! Defines rpc queries exposed by the ibc module
use std::cell::RefCell;
use std::rc::Rc;

use ibc::proto::core::channel::v1::{
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
use ibc::proto::core::client::v1::{
    QueryClientStateRequest, QueryClientStateResponse, QueryClientStatesRequest,
    QueryClientStatesResponse, QueryClientStatusRequest, QueryClientStatusResponse,
    QueryConsensusStateHeightsRequest, QueryConsensusStateHeightsResponse,
    QueryConsensusStateRequest, QueryConsensusStateResponse, QueryConsensusStatesRequest,
    QueryConsensusStatesResponse,
};
use ibc::proto::core::connection::v1::{
    QueryClientConnectionsRequest, QueryClientConnectionsResponse,
    QueryConnectionClientStateRequest, QueryConnectionClientStateResponse,
    QueryConnectionConsensusStateRequest, QueryConnectionConsensusStateResponse,
    QueryConnectionParamsRequest, QueryConnectionParamsResponse, QueryConnectionRequest,
    QueryConnectionResponse, QueryConnectionsRequest, QueryConnectionsResponse,
};
use ibc_query::core::channel::{
    query_channel, query_channel_client_state, query_channel_consensus_state, query_channels,
    query_connection_channels, query_next_sequence_receive, query_packet_acknowledgement,
    query_packet_acknowledgements, query_packet_commitment, query_packet_commitments,
    query_packet_receipt, query_unreceived_acks, query_unreceived_packets,
};
use ibc_query::core::client::{
    query_client_state, query_client_states, query_client_status, query_consensus_state,
    query_consensus_state_heights, query_consensus_states,
};
use ibc_query::core::connection::{
    query_client_connections, query_connection, query_connection_client_state,
    query_connection_consensus_state, query_connection_params, query_connections,
};
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::{Context, DaSpec, WorkingSet};

use crate::context::IbcContext;
use crate::Ibc;

/// Structure returned by the `client_state` rpc method.
#[rpc_gen(client, server, namespace = "ibc")]
impl<C: Context, Da: DaSpec> Ibc<C, Da> {
    #[rpc_method(name = "clientState")]
    pub fn client_state(
        &self,
        request: QueryClientStateRequest,
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<QueryClientStateResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_client_state(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "clientStates")]
    pub fn client_states(
        &self,
        request: QueryClientStatesRequest,
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<QueryConsensusStateResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_consensus_state(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "consensusStates")]
    pub fn consensus_states(
        &self,
        request: QueryConsensusStatesRequest,
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<QueryClientStatusResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_client_status(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "connection")]
    pub fn connection(
        &self,
        request: QueryConnectionRequest,
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<QueryConnectionChannelsResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_connection_channels(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "channelClientState")]
    pub fn query_channel_client_state(
        &self,
        request: QueryChannelClientStateRequest,
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<QueryChannelClientStateResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_channel_client_state(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "channelConsensusState")]
    pub fn query_channel_consensus_state(
        &self,
        request: QueryChannelConsensusStateRequest,
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<QueryPacketCommitmentResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_packet_commitment(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "packetCommitments")]
    pub fn packet_commitments(
        &self,
        request: QueryPacketCommitmentsRequest,
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
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
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<QueryNextSequenceReceiveResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_next_sequence_receive(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }
}

/// Creates an jsonrpsee error object
fn to_jsonrpsee_error(err: impl ToString) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(
        jsonrpsee::types::error::UNKNOWN_ERROR_CODE,
        err.to_string(),
        None::<String>,
    )
}
