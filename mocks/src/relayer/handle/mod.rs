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
    fn query(&self, request: QueryReq) -> QueryResp;

    fn send_msg(&self, msg: Vec<Any>) -> Vec<IbcEvent>;
}

impl<Ctx> Handle for Ctx
where
    Ctx: QueryService,
{
    fn query(&self, request: QueryReq) -> QueryResp {
        Ctx::service(self).query(request)
    }

    fn send_msg(&self, msg: Vec<Any>) -> Vec<IbcEvent> {
        Ctx::service(self).send_msg(msg)
    }
}

pub enum QueryReq {
    ChainId,
    ClientCounter,
    HostHeight,
    HostConsensusState(Height),
    Header(Height, Height),
    ClientState(ClientId),
    NextSeqSend(SeqSendPath),
    ValueWithProof(Path, Height),
}

pub enum QueryResp {
    ChainId(ChainId),
    ClientCounter(u64),
    HostHeight(Height),
    HostConsensusState(Any),
    Header(Any),
    ClientState(Any),
    NextSeqSend(Sequence),
    ValueWithProof(Vec<u8>, Vec<u8>),
}
