//! Defines JSON RPC methods exposed by the ibc module
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use borsh::BorshSerialize;
use ibc_core::channel::types::proto::v1::{
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
use ibc_core::client::types::proto::v1::{
    QueryClientStateRequest, QueryClientStateResponse, QueryClientStatesRequest,
    QueryClientStatesResponse, QueryClientStatusRequest, QueryClientStatusResponse,
    QueryConsensusStateHeightsRequest, QueryConsensusStateHeightsResponse,
    QueryConsensusStateRequest, QueryConsensusStateResponse, QueryConsensusStatesRequest,
    QueryConsensusStatesResponse,
};
use ibc_core::connection::types::proto::v1::{
    QueryClientConnectionsRequest, QueryClientConnectionsResponse,
    QueryConnectionClientStateRequest, QueryConnectionClientStateResponse,
    QueryConnectionConsensusStateRequest, QueryConnectionConsensusStateResponse,
    QueryConnectionParamsRequest, QueryConnectionParamsResponse, QueryConnectionRequest,
    QueryConnectionResponse, QueryConnectionsRequest, QueryConnectionsResponse,
};
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::{ClientConsensusStatePath, CommitmentPath};
use ibc_core::host::ValidationContext;
use ibc_query::core::channel::{
    query_channel, query_channel_client_state, query_channel_consensus_state, query_channels,
    query_connection_channels, query_next_sequence_receive, query_packet_acknowledgement,
    query_packet_acknowledgements, query_packet_commitments, query_packet_receipt,
    query_unreceived_acks, query_unreceived_packets,
};
use ibc_query::core::client::{
    query_client_states, query_client_status, query_consensus_state_heights, query_consensus_states,
};
use ibc_query::core::connection::{
    query_client_connections, query_connection, query_connection_client_state,
    query_connection_consensus_state, query_connection_params, query_connections,
};
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::{Context, DaSpec, WorkingSet};
use sov_state::storage::{StateCodec, StateValueCodec, StorageKey};

use crate::clients::{AnyClientState, AnyConsensusState};
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
        let prefix = self.client_state_map.prefix();

        let codec = self.client_state_map.codec();

        let client_id =
            ClientId::from_str(request.client_id.as_str()).map_err(to_jsonrpsee_error)?;

        let key = StorageKey::new(prefix, &client_id, codec.key_codec());

        let value_with_proof = working_set.get_with_proof(key);

        let storage_value = value_with_proof.value.ok_or_else(|| {
            to_jsonrpsee_error(format!("Client state not found for client {client_id:?}"))
        })?;

        let client_state: AnyClientState = codec
            .try_decode_value(storage_value.value())
            .map_err(to_jsonrpsee_error)?;

        let proof = value_with_proof
            .proof
            .try_to_vec()
            .map_err(to_jsonrpsee_error)?;

        let ibc_ctx = IbcContext {
            ibc: self,
            context: None,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        let current_height = ibc_ctx.host_height().map_err(to_jsonrpsee_error)?;

        Ok(QueryClientStateResponse {
            client_state: Some(client_state.into()),
            proof,
            proof_height: Some(current_height.into()),
        })
    }

    #[rpc_method(name = "clientStates")]
    pub fn client_states(
        &self,
        request: QueryClientStatesRequest,
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<QueryClientStatesResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            context: None,
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
        let prefix = self.consensus_state_map.prefix();

        let codec = self.consensus_state_map.codec();

        let client_id =
            ClientId::from_str(request.client_id.as_str()).map_err(to_jsonrpsee_error)?;

        let path = ClientConsensusStatePath::new(
            client_id.clone(),
            request.revision_number,
            request.revision_height,
        );

        let key = StorageKey::new(prefix, &path, codec.key_codec());

        let value_with_proof = working_set.get_with_proof(key);

        let storage_value = value_with_proof.value.ok_or_else(|| {
            to_jsonrpsee_error(format!(
                "Consensus state not found for client {client_id:?}"
            ))
        })?;

        let consensus_state: AnyConsensusState = codec
            .try_decode_value(storage_value.value())
            .map_err(to_jsonrpsee_error)?;

        let proof = value_with_proof
            .proof
            .try_to_vec()
            .map_err(to_jsonrpsee_error)?;

        let ibc_ctx = IbcContext {
            ibc: self,
            context: None,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        let proof_height = ibc_ctx.host_height().map_err(to_jsonrpsee_error)?;

        Ok(QueryConsensusStateResponse {
            consensus_state: Some(consensus_state.into()),
            proof,
            proof_height: Some(proof_height.into()),
        })
    }

    #[rpc_method(name = "consensusStates")]
    pub fn consensus_states(
        &self,
        request: QueryConsensusStatesRequest,
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<QueryConsensusStatesResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            context: None,
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
            context: None,
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
            context: None,
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
            context: None,
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
            context: None,
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
            context: None,
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
            context: None,
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
            context: None,
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
            context: None,
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
            context: None,
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
            context: None,
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
            context: None,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_connection_channels(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "channelClientState")]
    pub fn channel_client_state(
        &self,
        request: QueryChannelClientStateRequest,
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<QueryChannelClientStateResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            context: None,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        query_channel_client_state(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "channelConsensusState")]
    pub fn channel_consensus_state(
        &self,
        request: QueryChannelConsensusStateRequest,
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<QueryChannelConsensusStateResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            context: None,
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
        let prefix = self.packet_commitment_map.prefix();

        let codec = self.packet_commitment_map.codec();

        let commitment_path = CommitmentPath::new(
            &request.port_id.parse().map_err(to_jsonrpsee_error)?,
            &request.channel_id.parse().map_err(to_jsonrpsee_error)?,
            request.sequence.into(),
        );

        let key = StorageKey::new(prefix, &commitment_path, codec.key_codec());

        let value_with_proof = working_set.get_with_proof(key);

        let storage_value = value_with_proof.value.ok_or_else(|| {
            to_jsonrpsee_error(format!(
                "Packet commitment not found for path {commitment_path:?}"
            ))
        })?;

        let proof = value_with_proof
            .proof
            .try_to_vec()
            .map_err(to_jsonrpsee_error)?;

        let ibc_ctx = IbcContext {
            ibc: self,
            context: None,
            working_set: Rc::new(RefCell::new(working_set)),
        };

        let current_height = ibc_ctx.host_height().map_err(to_jsonrpsee_error)?;

        Ok(QueryPacketCommitmentResponse {
            commitment: storage_value.value().to_vec(),
            proof,
            proof_height: Some(current_height.into()),
        })
    }

    #[rpc_method(name = "packetCommitments")]
    pub fn packet_commitments(
        &self,
        request: QueryPacketCommitmentsRequest,
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<QueryPacketCommitmentsResponse> {
        let ibc_ctx = IbcContext {
            ibc: self,
            context: None,
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
            context: None,
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
            context: None,
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
            context: None,
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
            context: None,
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
            context: None,
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
            context: None,
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
