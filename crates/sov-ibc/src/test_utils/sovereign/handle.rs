use std::str::FromStr;

use ibc::applications::transfer::Memo;
use ibc::clients::ics07_tendermint::header::Header;
use ibc::core::events::IbcEvent;
use ibc::core::ics04_channel::timeout::TimeoutHeight;
use ibc::core::ics24_host::identifier::{ChainId, ChannelId, PortId};
use ibc::core::timestamp::Timestamp;
use ibc::{Any, Height, Signer};
use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::{Context, DaSpec, Module, Spec};
use sov_rollup_interface::mocks::MockDaSpec;

use super::app::TestApp;
use crate::applications::transfer::call::SDKTokenTransfer;
use crate::call::CallMessage;
use crate::clients::AnyConsensusState;
use crate::context::IbcExecutionContext;
use crate::test_utils::relayer::context::ChainContext;
use crate::test_utils::relayer::handle::Handle;

impl<'a, C, Da> Handle for TestApp<'a, C, Da>
where
    C: Context,
    Da: DaSpec + Clone,
{
    type IbcContext = IbcExecutionContext<'a, C, Da>;

    type Header = Header;

    type Message = CallMessage<C>;

    type Event = IbcEvent;

    fn query_chain_id(&self) -> &ChainId {
        self.chain_id()
    }

    // TODO: Can implement this when we have enough information about the header of SDK chains
    fn query_header(&self, target_height: &Height, trusted_height: &Height) -> Header {
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

impl ChainContext<TestApp<'_, DefaultContext, MockDaSpec>> {
    /// Builds a sdk token transfer message wrapped in a `CallMessage` with the given amount
    /// Note: keep the amount value lower than the initial balance of the sender address
    pub fn build_sdk_transfer(
        &self,
        token: <DefaultContext as Spec>::Address,
        sender: Signer,
        receiver: Signer,
        amount: u64,
    ) -> CallMessage<DefaultContext> {
        let msg_transfer = SDKTokenTransfer {
            port_id_on_a: PortId::transfer(),
            chan_id_on_a: ChannelId::default(),
            timeout_height_on_b: TimeoutHeight::no_timeout(),
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
