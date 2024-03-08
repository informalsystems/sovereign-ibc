use std::fmt::Debug;

use async_trait::async_trait;
use basecoin::helper::ibc;
use basecoin::store::context::ProvableStore;
use ibc_client_tendermint::types::Header;
use ibc_core::client::context::ClientValidationContext;
use ibc_core::handler::types::events::IbcEvent;
use ibc_core::host::types::path::ClientConsensusStatePath;
use ibc_core::host::ValidationContext;
use ibc_core::primitives::proto::Any;
use ibc_core_host_cosmos::IBC_QUERY_PATH;
use tracing::info;

use crate::cosmos::MockCosmosChain;
use crate::relayer::handle::{Handle, QueryReq, QueryResp};
use crate::utils::wait_for_block;

#[async_trait]
impl<S: ProvableStore + Debug + Default> Handle for MockCosmosChain<S> {
    type Message = Any;

    async fn query(&self, request: QueryReq) -> QueryResp {
        info!("cosmos: querying app with {:?}", request);

        match request {
            QueryReq::ChainId => QueryResp::ChainId(self.chain_id().clone()),
            QueryReq::HostHeight => QueryResp::HostHeight(self.ibc_ctx().host_height().unwrap()),
            QueryReq::HostConsensusState(height) => QueryResp::HostConsensusState(
                self.ibc_ctx().host_consensus_state(&height).unwrap().into(),
            ),
            QueryReq::ClientCounter => {
                QueryResp::ClientCounter(self.ibc_ctx().client_counter().unwrap())
            }
            QueryReq::ClientState(client_id) => QueryResp::ClientState(
                self.ibc_ctx()
                    .get_client_validation_context()
                    .client_state(&client_id)
                    .unwrap()
                    .into(),
            ),
            QueryReq::ConsensusState(client_id, height) => QueryResp::ConsensusState(
                self.ibc_ctx()
                    .get_client_validation_context()
                    .consensus_state(&ClientConsensusStatePath::new(
                        client_id,
                        height.revision_number(),
                        height.revision_height(),
                    ))
                    .unwrap()
                    .into(),
            ),
            QueryReq::NextSeqSend(path) => {
                QueryResp::NextSeqSend(self.ibc_ctx().get_next_sequence_send(&path).unwrap())
            }
            QueryReq::Header(target_height, trusted_height) => {
                let blocks = self.core.blocks();

                let revision_height = target_height.revision_height() as usize;

                if revision_height > blocks.len() {
                    panic!("block index out of bounds");
                }

                let target_block = blocks[revision_height - 1].clone();

                let header = Header {
                    signed_header: target_block.signed_header,
                    validator_set: target_block.validators,
                    trusted_height,
                    trusted_next_validator_set: target_block.next_validators,
                };

                QueryResp::Header(header.into())
            }
            QueryReq::ValueWithProof(path, height) => {
                let (value, proof) = self.query(
                    path.to_string().as_bytes().to_vec(),
                    IBC_QUERY_PATH.to_string(),
                    &height,
                );

                QueryResp::ValueWithProof(value, proof.into())
            }
        }
    }

    async fn submit_msgs(&self, msgs: Vec<Any>) -> Vec<IbcEvent> {
        info!("cosmos: submitting messages: {msgs:?}");

        let events = msgs
            .into_iter()
            .flat_map(|msg| ibc(self.app.clone()).process_message(msg).unwrap())
            .collect();

        wait_for_block().await;

        events
    }
}
