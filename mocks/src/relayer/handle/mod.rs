mod cosmos;
#[cfg(feature = "native")]
mod rollup;

use std::sync::Arc;

use async_trait::async_trait;
use ibc_core::client::types::Height;
use ibc_core::handler::types::events::IbcEvent;
use ibc_core::host::types::identifiers::{ChainId, ClientId, Sequence};
use ibc_core::host::types::path::{Path, SeqSendPath};
use ibc_core::primitives::proto::Any;

/// Defines the interface that empowers a chain context with the ability to
/// query different states of a chain.
pub trait QueryService: Send + Sync {
    type E: Handle;

    fn service(&self) -> &Arc<Self::E>;
}

/// Defines the interface that enables a mock chain to provide query endpoints.
#[async_trait]
pub trait Handle: Send + Sync {
    type Message: Send + Sync;

    async fn query(&self, request: QueryReq) -> QueryResp;

    async fn submit_msgs(&self, msg: Vec<Self::Message>) -> Vec<IbcEvent>;
}

#[async_trait]
impl<Ctx> Handle for Ctx
where
    Ctx: QueryService,
{
    type Message = <<Ctx as QueryService>::E as Handle>::Message;

    async fn query(&self, request: QueryReq) -> QueryResp {
        Ctx::service(self).query(request).await
    }

    async fn submit_msgs(&self, msgs: Vec<Self::Message>) -> Vec<IbcEvent> {
        Ctx::service(self).submit_msgs(msgs).await
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
