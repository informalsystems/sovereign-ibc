use std::fmt::Debug;

use basecoin_app::modules::ibc::{AnyConsensusState, IbcContext};
use basecoin_store::context::ProvableStore;
use basecoin_store::impls::RevertibleStore;
use ibc::clients::ics07_tendermint::header::Header;
use ibc::core::events::IbcEvent;
use ibc::core::ics24_host::identifier::ChainId;
use ibc::proto::Any;
use ibc::Height;

use super::app::MockCosmosChain;
use crate::test_utils::relayer::handle::Handle;

impl<S: ProvableStore + Debug + Default> Handle for MockCosmosChain<S> {
    type IbcContext = IbcContext<RevertibleStore<S>>;

    type Header = Header;

    type Message = Any;

    type Event = IbcEvent;

    fn query_chain_id(&self) -> &ChainId {
        self.chain_id()
    }

    fn query_header(&self, target_height: &Height, trusted_height: &Height) -> Header {
        let blocks = self.get_blocks();

        let revision_height = target_height.revision_height() as usize;

        if revision_height > blocks.len() {
            panic!("block index out of bounds");
        }

        let target_block = blocks[revision_height - 1].clone();

        Header {
            signed_header: target_block.signed_header,
            validator_set: target_block.validators,
            trusted_height: *trusted_height,
            trusted_next_validator_set: target_block.next_validators,
        }
    }

    fn query_ibc(&self) -> Self::IbcContext {
        self.ibc_ctx()
    }

    fn consensus_state_to_any(&self, consensus_state: AnyConsensusState) -> Any {
        let AnyConsensusState::Tendermint(cs) = consensus_state;
        cs.into()
    }

    fn send_msg(&self, msg: Vec<Self::Message>) -> Vec<Self::Event> {
        let mut events = vec![];

        for msg in msg {
            let event = self.app.ibc().process_message(msg).unwrap();
            events.extend(event);
        }

        events
    }
}
