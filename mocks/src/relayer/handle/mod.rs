mod cosmos;
#[cfg(feature = "native")]
mod rollup;

pub use cosmos::*;
use ibc_core::client::types::Height;
use ibc_core::handler::types::events::IbcEvent;
use ibc_core::host::types::identifiers::{ChainId, ClientId, Sequence};
use ibc_core::host::types::path::{Path, SeqSendPath};
use ibc_core::primitives::proto::Any;
#[cfg(feature = "native")]
pub use rollup::*;

/// Defines the interface that empowers a chain context with the ability to
/// query different states of a chain.
pub trait QueryService {
    type E: Handle;

    fn service(&self) -> &Self::E;
}

/// Defines the interface that enables a mock chain to provide query endpoints.
pub trait Handle {
    type Message;

    fn query(&self, request: QueryReq) -> QueryResp;

    fn submit_msg(&self, msg: Vec<Self::Message>) -> Vec<IbcEvent>;
}

impl<Ctx> Handle for Ctx
where
    Ctx: QueryService,
{
    type Message = <<Ctx as QueryService>::E as Handle>::Message;

    fn query(&self, request: QueryReq) -> QueryResp {
        Ctx::service(self).query(request)
    }

    fn submit_msg(&self, msg: Vec<Self::Message>) -> Vec<IbcEvent> {
        Ctx::service(self).submit_msg(msg)
    }
}

/// Defines the different types of queries that can be made to a chain.
#[derive(Debug)]
pub enum QueryReq {
    ChainId,
    ClientCounter,
    HostHeight,
    HostConsensusState(Height),
    Header(Height, Height),
    ClientState(ClientId),
    ConsensusState(ClientId, Height),
    NextSeqSend(SeqSendPath),
    ValueWithProof(Path, Height),
}

/// Defines the different types of responses that can be returned from querying
#[derive(Debug)]
pub enum QueryResp {
    ChainId(ChainId),
    ClientCounter(u64),
    HostHeight(Height),
    HostConsensusState(Any),
    Header(Any),
    ClientState(Any),
    ConsensusState(Any),
    NextSeqSend(Sequence),
    ValueWithProof(Vec<u8>, Vec<u8>),
}
