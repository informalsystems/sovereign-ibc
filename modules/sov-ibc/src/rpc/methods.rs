//! Defines JSON RPC methods exposed by the ibc module
use std::cell::RefCell;
use std::rc::Rc;

use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::host::ValidationContext;
use ibc_query::core::channel::{
    query_channel_consensus_state, query_channels, query_connection_channels,
    query_packet_acknowledgements, query_packet_commitments, query_unreceived_acks,
    query_unreceived_packets, QueryChannelClientStateRequest, QueryChannelClientStateResponse,
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
    query_consensus_states, IdentifiedClientState, QueryClientStateRequest,
    QueryClientStateResponse, QueryClientStatesRequest, QueryClientStatesResponse,
    QueryClientStatusRequest, QueryClientStatusResponse, QueryConsensusStateHeightsRequest,
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
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::{Spec, WorkingSet};

use crate::context::IbcContext;
use crate::helpers::{to_jsonrpsee_error, WithProof, WithoutProof};
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
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let (client_state, proof) = ibc_ctx.dyn_client_state::<WithProof<_>>(&request.client_id);

        Ok(QueryClientStateResponse::new(
            client_state
                .ok_or_else(|| {
                    to_jsonrpsee_error(format!(
                        "Client state not found for client {:?}",
                        request.client_id
                    ))
                })?
                .into(),
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
        let proof_height = self.determine_query_height(request.query_height, working_set)?;
        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let consensus_height = request.consensus_height.ok_or_else(|| {
            to_jsonrpsee_error("Consensus height is required for querying consensus state")
        })?;

        let (consensus_state, proof) = ibc_ctx.dyn_client_consensus_state::<WithProof<_>>(
            &request.client_id,
            consensus_height.revision_number(),
            consensus_height.revision_height(),
        );

        let proof_height = ibc_ctx.host_height().map_err(to_jsonrpsee_error)?;

        Ok(QueryConsensusStateResponse::new(
            consensus_state
                .ok_or_else(|| {
                    to_jsonrpsee_error(format!(
                        "Consensus state not found for client {:?} at height {:?}",
                        request.client_id, consensus_height
                    ))
                })?
                .into(),
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
        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let (upgraded_client_state, proof) =
            ibc_ctx.dyn_upgraded_client_state::<WithProof<_>>(proof_height.revision_height());

        Ok(QueryUpgradedClientStateResponse::new(
            upgraded_client_state
                .ok_or_else(|| {
                    to_jsonrpsee_error(format!(
                        "Upgraded client state not found at height {proof_height:?}"
                    ))
                })?
                .into(),
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
        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let (upgraded_consensus_state, proof) =
            ibc_ctx.dyn_upgraded_consensus_state::<WithProof<_>>(proof_height.revision_height());

        Ok(QueryUpgradedConsensusStateResponse::new(
            upgraded_consensus_state
                .ok_or_else(|| {
                    to_jsonrpsee_error(format!(
                        "Upgraded consensus state not found at height {proof_height:?}"
                    ))
                })?
                .into(),
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
        let proof_height = self.determine_query_height(request.query_height, working_set)?;
        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let (connection_end, proof) =
            ibc_ctx.dyn_connection_end::<WithProof<_>>(&request.connection_id);

        Ok(QueryConnectionResponse::new(
            connection_end.ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Connection not found for connection id {:?}",
                    request.connection_id
                ))
            })?,
            proof,
            proof_height,
        ))
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
        let proof_height = self.determine_query_height(None, working_set)?;
        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let (client_connections, proof) =
            ibc_ctx.dyn_client_connections::<WithProof<_>>(&request.client_id);

        Ok(QueryClientConnectionsResponse::new(
            client_connections.ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Client connections not found for client id {:?}",
                    request.client_id
                ))
            })?,
            proof,
            proof_height,
        ))
    }

    #[rpc_method(name = "connectionClientState")]
    pub fn connection_client_state(
        &self,
        request: QueryConnectionClientStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConnectionClientStateResponse> {
        let proof_height = self.determine_query_height(request.query_height, working_set)?;
        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let connection_end = ibc_ctx
            .dyn_connection_end::<WithoutProof<_>>(&request.connection_id)
            .ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Connection not found for connection id {:?}",
                    request.connection_id
                ))
            })?;

        let (client_state, proof) =
            ibc_ctx.dyn_client_state::<WithProof<_>>(connection_end.client_id());

        Ok(QueryConnectionClientStateResponse::new(
            IdentifiedClientState::new(
                connection_end.client_id().clone(),
                client_state
                    .ok_or_else(|| {
                        to_jsonrpsee_error(format!(
                            "Client state not found for connection {:?}",
                            request.connection_id
                        ))
                    })?
                    .into(),
            ),
            proof,
            proof_height,
        ))
    }

    #[rpc_method(name = "connectionConsensusState")]
    pub fn connection_consensus_state(
        &self,
        request: QueryConnectionConsensusStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryConnectionConsensusStateResponse> {
        let proof_height = self.determine_query_height(request.query_height, working_set)?;
        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let connection_end = ibc_ctx
            .dyn_connection_end::<WithoutProof<_>>(&request.connection_id)
            .ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Connection not found for connection id {:?}",
                    request.connection_id
                ))
            })?;

        let (consensus_state, proof) = ibc_ctx.dyn_client_consensus_state::<WithProof<_>>(
            connection_end.client_id(),
            request.height.revision_number(),
            request.height.revision_height(),
        );

        Ok(QueryConnectionConsensusStateResponse::new(
            consensus_state
                .ok_or_else(|| {
                    to_jsonrpsee_error(format!(
                        "Consensus state not found for connection {:?}",
                        request.connection_id
                    ))
                })?
                .into(),
            connection_end.client_id().clone(),
            proof,
            proof_height,
        ))
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
        let proof_height = self.determine_query_height(request.query_height, working_set)?;
        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let (channel_end, proof) =
            ibc_ctx.dyn_channel_end::<WithProof<_>>(&request.port_id, &request.channel_id);

        Ok(QueryChannelResponse::new(
            channel_end.ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Channel not found for port id {:?} and channel id {:?}",
                    request.port_id, request.channel_id
                ))
            })?,
            proof,
            proof_height,
        ))
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
        let proof_height = self.determine_query_height(request.query_height, working_set)?;
        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let channel_end = ibc_ctx
            .dyn_channel_end::<WithoutProof<_>>(&request.port_id, &request.channel_id)
            .ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Channel not found for port id {:?} and channel id {:?}",
                    request.port_id, request.channel_id
                ))
            })?;

        let connection_id = channel_end.connection_hops().first().ok_or_else(|| {
            to_jsonrpsee_error(format!(
                "ConnectionId not found for channel {:?}",
                request.channel_id
            ))
        })?;

        let connection_end = ibc_ctx
            .dyn_connection_end::<WithoutProof<_>>(connection_id)
            .ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "ConnectionEnd not found for channel {:?}",
                    request.channel_id
                ))
            })?;

        let (client_state, proof) =
            ibc_ctx.dyn_client_state::<WithProof<_>>(connection_end.client_id());

        Ok(QueryChannelClientStateResponse::new(
            IdentifiedClientState::new(
                connection_end.client_id().clone(),
                client_state
                    .ok_or_else(|| {
                        to_jsonrpsee_error(format!(
                            "Client state not found for channel {:?}",
                            request.channel_id
                        ))
                    })?
                    .into(),
            ),
            proof,
            proof_height,
        ))
    }

    #[rpc_method(name = "channelConsensusState")]
    pub fn channel_consensus_state(
        &self,
        request: QueryChannelConsensusStateRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryChannelConsensusStateResponse> {
        let proof_height = self.determine_query_height(request.query_height, working_set)?;
        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let channel_end = ibc_ctx
            .dyn_channel_end::<WithoutProof<_>>(&request.port_id, &request.channel_id)
            .ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Channel not found for port id {:?} and channel id {:?}",
                    request.port_id, request.channel_id
                ))
            })?;

        let connection_id = channel_end.connection_hops().first().ok_or_else(|| {
            to_jsonrpsee_error(format!(
                "ConnectionId not found for channel {:?}",
                request.channel_id
            ))
        })?;

        let connection_end = ibc_ctx
            .dyn_connection_end::<WithoutProof<_>>(connection_id)
            .ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "ConnectionEnd not found for channel {:?}",
                    request.channel_id
                ))
            })?;

        let client_state = ibc_ctx
            .dyn_client_state::<WithoutProof<_>>(connection_end.client_id())
            .ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Client state not found for channel {:?}",
                    request.channel_id
                ))
            })?;

        let client_latest_height = client_state.latest_height();

        let (consensus_state, proof) = ibc_ctx.dyn_client_consensus_state::<WithProof<_>>(
            connection_end.client_id(),
            client_latest_height.revision_number(),
            client_latest_height.revision_height(),
        );

        query_channel_consensus_state(&ibc_ctx, &request).map_err(to_jsonrpsee_error)
    }

    #[rpc_method(name = "packetCommitment")]
    pub fn packet_commitment(
        &self,
        request: QueryPacketCommitmentRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryPacketCommitmentResponse> {
        let proof_height = self.determine_query_height(request.query_height, working_set)?;
        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let (commitment, proof) = ibc_ctx.dyn_packet_commitment::<WithProof<_>>(
            &request.port_id,
            &request.channel_id,
            request.sequence,
        );

        Ok(QueryPacketCommitmentResponse::new(
            commitment.ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Packet commitment not found for port id {:?}, channel id {:?} and sequence {:?}",
                    request.port_id, request.channel_id, request.sequence
                ))
            })?,
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
        let proof_height = self.determine_query_height(request.query_height, working_set)?;
        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let (receipt, proof) = ibc_ctx.dyn_packet_receipt::<WithProof<_>>(
            &request.port_id,
            &request.channel_id,
            request.sequence,
        );

        // packet_receipt_map models a set using constant unit value.
        // when the key (doesn't) exists in the map,
        // the receipt is (not) present and returns a (non) membership proof
        Ok(QueryPacketReceiptResponse::new(
            receipt.is_some(),
            proof,
            proof_height,
        ))
    }

    #[rpc_method(name = "packetAcknowledgement")]
    pub fn packet_acknowledgement(
        &self,
        request: QueryPacketAcknowledgementRequest,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<QueryPacketAcknowledgementResponse> {
        let proof_height = self.determine_query_height(request.query_height, working_set)?;
        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let (acknowledgement, proof) = ibc_ctx.dyn_packet_acknowledgement::<WithProof<_>>(
            &request.port_id,
            &request.channel_id,
            request.sequence,
        );

        Ok(QueryPacketAcknowledgementResponse::new(
            acknowledgement.ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Packet acknowledgement not found for port id {:?}, channel id {:?} and sequence {:?}",
                    request.port_id, request.channel_id, request.sequence
                ))
            })?,
            proof,
            proof_height,
        ))
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
        let proof_height = self.determine_query_height(request.query_height, working_set)?;
        let mut archival_working_set = working_set.get_archival_at(proof_height.revision_height());
        let ibc_ctx = IbcContext::new(self, Rc::new(RefCell::new(&mut archival_working_set)));

        let (sequence, proof) =
            ibc_ctx.dyn_recv_sequence::<WithProof<_>>(&request.port_id, &request.channel_id);

        Ok(QueryNextSequenceReceiveResponse::new(
            sequence.ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Next sequence receive not found for port id {:?} and channel id {:?}",
                    request.port_id, request.channel_id
                ))
            })?,
            proof,
            proof_height,
        ))
    }
}
