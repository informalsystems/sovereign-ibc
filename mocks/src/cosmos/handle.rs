use std::fmt::Debug;
use std::str::FromStr;

use basecoin_app::modules::ibc::{AnyConsensusState, IbcContext};
use basecoin_store::context::ProvableStore;
use basecoin_store::impls::RevertibleStore;
use ibc_app_transfer::types::msgs::transfer::MsgTransfer;
use ibc_app_transfer::types::packet::PacketData;
use ibc_app_transfer::types::{Coin, Memo, PrefixedDenom};
use ibc_client_tendermint::types::Header;
use ibc_core::channel::types::timeout::TimeoutHeight;
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::CommitmentProofBytes;
use ibc_core::handler::types::events::IbcEvent;
use ibc_core::host::types::identifiers::{ChainId, ChannelId, PortId};
use ibc_core::primitives::proto::Any;
use ibc_core::primitives::{Signer, Timestamp};

use super::app::MockCosmosChain;
use crate::relayer::context::ChainContext;
use crate::relayer::handle::Handle;

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

    fn query(
        &self,
        data: Vec<u8>,
        path: String,
        height: &Height,
    ) -> (Vec<u8>, CommitmentProofBytes) {
        self.sync_query(data, path, height)
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

impl<S: ProvableStore + Debug + Default> ChainContext<MockCosmosChain<S>> {
    /// Builds a CosmosChain token transfer message; serialized to Any
    /// Note: keep the amount value lower than the initial balance of the sender address
    pub fn build_token_transfer(
        &self,
        denom: PrefixedDenom,
        sender: Signer,
        receiver: Signer,
        amount: u64,
    ) -> MsgTransfer {
        let packet_data = PacketData {
            token: Coin {
                denom,
                amount: amount.into(),
            },
            sender,
            receiver,
            memo: Memo::from_str("").unwrap(),
        };

        MsgTransfer {
            port_id_on_a: PortId::transfer(),
            chan_id_on_a: ChannelId::default(),
            packet_data,
            timeout_height_on_b: TimeoutHeight::At(Height::new(1, 200).unwrap()),
            timeout_timestamp_on_b: Timestamp::none(),
        }
    }
}
