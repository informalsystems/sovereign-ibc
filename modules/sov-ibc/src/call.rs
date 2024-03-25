use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use anyhow::{bail, Result};
use ibc_app_transfer::handler::send_transfer;
use ibc_app_transfer::types::msgs::transfer::MsgTransfer;
use ibc_core::entrypoint::dispatch;
use ibc_core::handler::types::msgs::MsgEnvelope;
use ibc_core::primitives::proto::Any;
use sov_ibc_transfer::context::IbcTransferContext;
use sov_modules_api::{CallResponse, Context, Spec, WorkingSet};
use tracing::info;

use crate::context::IbcContext;
use crate::router::IbcRouter;
use crate::Ibc;

#[cfg_attr(feature = "native", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq)]
pub enum CallMessage {
    Core(Any),

    Transfer(MsgTransfer),
}

impl<S: Spec> Ibc<S> {
    pub(crate) fn process_core_message(
        &self,
        msg: Any,
        context: Context<S>,
        working_set: &mut WorkingSet<S>,
    ) -> Result<CallResponse> {
        let msg_envelope = MsgEnvelope::try_from(msg).map_err(|e| {
            anyhow::anyhow!("Failed to convert Any to MsgEnvelope: {}", e.to_string())
        })?;

        info!(
            "Processing IBC core message: {:?} at visible slot number: {:?}",
            msg_envelope,
            context.visible_slot_number()
        );

        let shared_working_set = Rc::new(RefCell::new(working_set));

        let mut ibc_ctx = IbcContext {
            ibc: self,
            working_set: shared_working_set.clone(),
        };

        ibc_ctx.height_sanity_check(context.visible_slot_number())?;

        let mut router = IbcRouter::new(self, context.clone(), shared_working_set);

        match dispatch(&mut ibc_ctx, &mut router, msg_envelope) {
            Ok(_) => Ok(CallResponse::default()),
            Err(e) => bail!(e.to_string()),
        }
    }

    pub(crate) fn transfer(
        &self,
        msg_transfer: MsgTransfer,
        context: Context<S>,
        working_set: &mut WorkingSet<S>,
    ) -> Result<CallResponse> {
        info!(
            "Processing IBC transfer message: {:?} at visible_slot_number: {:?}",
            msg_transfer,
            context.visible_slot_number()
        );

        let shared_working_set = Rc::new(RefCell::new(working_set));

        let mut ibc_ctx = IbcContext {
            ibc: self,
            working_set: shared_working_set.clone(),
        };

        ibc_ctx.height_sanity_check(context.visible_slot_number())?;

        let mut transfer_ctx =
            IbcTransferContext::new(self.transfer.clone(), context, shared_working_set.clone());

        send_transfer(&mut ibc_ctx, &mut transfer_ctx, msg_transfer)?;

        Ok(sov_modules_api::CallResponse::default())
    }
}
