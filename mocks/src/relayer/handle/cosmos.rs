use std::fmt::Debug;

use basecoin_store::context::ProvableStore;
use ibc_client_tendermint::types::Header;
use ibc_core::handler::types::events::IbcEvent;
use ibc_core::host::ValidationContext;
use ibc_core::primitives::proto::Any;
use ibc_core_host_cosmos::IBC_QUERY_PATH;

use crate::cosmos::app::MockCosmosChain;
use crate::relayer::handle::{Handle, QueryReq, QueryResp};

impl<S: ProvableStore + Debug + Default> Handle for MockCosmosChain<S> {
    fn query(&self, request: QueryReq) -> QueryResp {
        match request {
            QueryReq::ChainId => QueryResp::ChainId(self.chain_id().clone()),
            QueryReq::HostHeight => QueryResp::HostHeight(self.ibc_ctx().host_height().unwrap()),
            QueryReq::HostConsensusState(height) => QueryResp::HostConsensusState(
                self.ibc_ctx().host_consensus_state(&height).unwrap().into(),
            ),
            QueryReq::ClientCounter => {
                QueryResp::ClientCounter(self.ibc_ctx().client_counter().unwrap())
            }
            QueryReq::ClientState(client_id) => {
                QueryResp::ClientState(self.ibc_ctx().client_state(&client_id).unwrap().into())
            }
            QueryReq::NextSeqSend(path) => {
                QueryResp::NextSeqSend(self.ibc_ctx().get_next_sequence_send(&path).unwrap())
            }
            QueryReq::Header(target_height, trusted_height) => {
                let blocks = self.get_blocks();

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

    fn send_msg(&self, msgs: Vec<Any>) -> Vec<IbcEvent> {
        msgs.into_iter()
            .flat_map(|msg| self.app.ibc().process_message(msg).unwrap())
            .collect()
    }
}
