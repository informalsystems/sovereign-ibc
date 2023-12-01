use ibc_client_tendermint::types::Header;
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::CommitmentProofBytes;
use ibc_core::handler::types::events::IbcEvent;
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::primitives::proto::Any;
use sov_ibc::call::CallMessage;
use sov_ibc::clients::AnyConsensusState;
use sov_ibc::context::IbcContext;
use sov_modules_api::{Context, DaSpec, Module};

use super::app::TestApp;
use crate::relayer::handle::Handle;

impl<'ws, C, Da> Handle for TestApp<'ws, C, Da>
where
    C: Context,
    Da: DaSpec + Clone,
{
    type IbcContext = IbcContext<'ws, C, Da>;

    type Header = Header;

    type Message = CallMessage<C>;

    type Event = IbcEvent;

    fn query_chain_id(&self) -> &ChainId {
        self.chain_id()
    }

    // TODO: Can implement this when we have enough information about the header of SDK chains
    fn query_header(&self, _target_height: &Height, _trusted_height: &Height) -> Header {
        unimplemented!()
    }

    fn query_ibc(&self) -> Self::IbcContext {
        self.ibc_ctx()
    }

    fn query(
        &self,
        _data: Vec<u8>,
        _path: String,
        _height: &Height,
    ) -> (Vec<u8>, CommitmentProofBytes) {
        unimplemented!()
    }

    fn send_msg(&self, msg: Vec<Self::Message>) -> Vec<Self::Event> {
        for m in msg {
            self.ibc_ctx()
                .ibc
                .call(m, self.sdk_ctx(), *self.working_set().borrow_mut())
                .unwrap();
        }
        vec![]
    }

    fn consensus_state_to_any(&self, consensus_state: AnyConsensusState) -> Any {
        let AnyConsensusState::Tendermint(cs) = consensus_state;
        cs.into()
    }
}
