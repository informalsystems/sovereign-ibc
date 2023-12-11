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
use sov_modules_api::{CallResponse, Context, DaSpec, WorkingSet};
use thiserror::Error;

use crate::context::IbcContext;
use crate::router::IbcRouter;
use crate::Ibc;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq)]
pub enum CallMessage {
    Core(Any),

    Transfer(MsgTransfer),
}

/// Example of a custom error.
#[derive(Debug, Error)]
enum SetValueError {}

impl<C: Context, Da: DaSpec> Ibc<C, Da> {
    pub(crate) fn process_core_message(
        &self,
        msg: Any,
        context: C,
        working_set: &mut WorkingSet<C>,
    ) -> Result<CallResponse> {
        let shared_working_set = Rc::new(RefCell::new(working_set));

        let mut execution_context = IbcContext {
            ibc: self,
            working_set: shared_working_set.clone(),
        };

        let mut router = IbcRouter::new(self, context, shared_working_set);

        let msg_envelope = MsgEnvelope::try_from(msg).map_err(|e| {
            anyhow::anyhow!("Failed to convert Any to MsgEnvelope: {}", e.to_string())
        })?;

        match dispatch(&mut execution_context, &mut router, msg_envelope) {
            Ok(_) => Ok(CallResponse::default()),
            Err(e) => bail!(e.to_string()),
        }
    }

    pub(crate) fn transfer(
        &self,
        msg_transfer: MsgTransfer,
        context: C,
        working_set: &mut WorkingSet<C>,
    ) -> Result<CallResponse> {
        let shared_working_set = Rc::new(RefCell::new(working_set));

        let mut ibc_ctx = IbcContext {
            ibc: self,
            working_set: shared_working_set.clone(),
        };

        let mut transfer_ctx =
            IbcTransferContext::new(self.transfer.clone(), context, shared_working_set.clone());

        send_transfer(&mut ibc_ctx, &mut transfer_ctx, msg_transfer)?;

        Ok(sov_modules_api::CallResponse::default())
    }
}
