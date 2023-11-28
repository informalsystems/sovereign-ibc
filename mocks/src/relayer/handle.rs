use std::sync::Arc;

use ibc_core::client::types::Height;
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::host::ValidationContext;
use ibc_core::primitives::proto::Any;

/// Defines the interface that empowers a chain context with the ability to
/// query different states of a chain.
pub trait QueryService {
    type E: Handle;

    fn service(&self) -> &Arc<Self::E>;
}

/// Defines the interface that enables a mock chain to provide query endpoints.
pub trait Handle {
    type IbcContext: ValidationContext;

    type Header: Into<Any>;

    type Message;

    type Event;

    fn query_chain_id(&self) -> &ChainId;

    fn query_header(&self, target_height: &Height, trusted_height: &Height) -> Self::Header;

    fn consensus_state_to_any(
        &self,
        cons_state: <<Self as Handle>::IbcContext as ValidationContext>::AnyConsensusState,
    ) -> Any;

    fn query_ibc(&self) -> Self::IbcContext;

    fn send_msg(&self, msg: Vec<Self::Message>) -> Vec<Self::Event>;
}

impl<Ctx> Handle for Ctx
where
    Ctx: QueryService,
{
    type IbcContext = <<Ctx as QueryService>::E as Handle>::IbcContext;

    type Header = <<Ctx as QueryService>::E as Handle>::Header;

    type Message = <<Ctx as QueryService>::E as Handle>::Message;

    type Event = <<Ctx as QueryService>::E as Handle>::Event;

    fn query_chain_id(&self) -> &ChainId {
        Ctx::service(self).query_chain_id()
    }

    fn query_header(&self, target_height: &Height, trusted_height: &Height) -> Self::Header {
        Ctx::service(self).query_header(target_height, trusted_height)
    }

    fn consensus_state_to_any(
        &self,
        cons_state: <<Self as Handle>::IbcContext as ValidationContext>::AnyConsensusState,
    ) -> Any {
        Ctx::service(self).consensus_state_to_any(cons_state)
    }

    fn query_ibc(&self) -> Self::IbcContext {
        Ctx::service(self).query_ibc()
    }

    fn send_msg(&self, msg: Vec<Self::Message>) -> Vec<Self::Event> {
        Ctx::service(self).send_msg(msg)
    }
}
