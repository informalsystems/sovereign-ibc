use std::str::FromStr;

use ibc::applications::transfer::Memo;
use ibc::clients::ics07_tendermint::header::Header;
use ibc::core::events::IbcEvent;
use ibc::core::ics04_channel::timeout::TimeoutHeight;
use ibc::core::ics24_host::identifier::{ChainId, ChannelId, PortId};
use ibc::core::timestamp::Timestamp;
use ibc::proto::Any;
use ibc::{Height, Signer};
use sov_ibc::call::CallMessage;
use sov_ibc::clients::AnyConsensusState;
use sov_ibc::context::IbcContext;
use sov_ibc_transfer::call::SDKTokenTransfer;
use sov_modules_api::{Context, DaSpec, Module, Spec};

use super::app::TestApp;
use crate::relayer::context::ChainContext;
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

impl<'ws, C: Context, Da: DaSpec + Clone> ChainContext<TestApp<'ws, C, Da>> {
    /// Builds a sdk token transfer message wrapped in a `CallMessage` with the given amount
    /// Note: keep the amount value lower than the initial balance of the sender address
    pub fn build_sdk_transfer(
        &self,
        token: <C as Spec>::Address,
        sender: Signer,
        receiver: Signer,
        amount: u64,
    ) -> CallMessage<C> {
        let msg_transfer = SDKTokenTransfer {
            port_id_on_a: PortId::transfer(),
            chan_id_on_a: ChannelId::default(),
            timeout_height_on_b: TimeoutHeight::At(Height::new(1, 200).unwrap()),
            timeout_timestamp_on_b: Timestamp::none(),
            token_address: token,
            amount,
            sender,
            receiver,
            memo: Memo::from_str("").unwrap(),
        };

        CallMessage::Transfer(msg_transfer)
    }
}
