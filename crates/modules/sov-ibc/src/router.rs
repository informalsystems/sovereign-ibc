use std::cell::RefCell;
use std::rc::Rc;

use ibc_app_transfer::types::{MODULE_ID_STR, PORT_ID_STR};
use ibc_core::host::types::identifiers::PortId;
use ibc_core::router::module::Module;
use ibc_core::router::router::Router;
use ibc_core::router::types::module::ModuleId;
use sov_ibc_transfer::context::IbcTransferContext;
use sov_modules_api::{Context, Spec, TxState};

use crate::Ibc;

pub struct IbcRouter<'ws, S: Spec, TS: TxState<S>> {
    pub transfer_ctx: IbcTransferContext<'ws, S, TS>,
}

impl<'ws, S: Spec, TS: TxState<S>> IbcRouter<'ws, S, TS> {
    pub fn new(
        ibc_mod: &Ibc<S>,
        sdk_context: Context<S>,
        working_set: Rc<RefCell<&'ws mut TS>>,
    ) -> IbcRouter<'ws, S, TS> {
        IbcRouter {
            transfer_ctx: IbcTransferContext::new(
                ibc_mod.transfer.clone(),
                sdk_context,
                working_set,
            ),
        }
    }
}

impl<'ws, S: Spec, TS: TxState<S>> Router for IbcRouter<'ws, S, TS> {
    fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module> {
        if *module_id == ModuleId::new(MODULE_ID_STR.to_string()) {
            Some(&self.transfer_ctx)
        } else {
            None
        }
    }

    fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
        if *module_id == ModuleId::new(MODULE_ID_STR.to_string()) {
            Some(&mut self.transfer_ctx)
        } else {
            None
        }
    }

    fn lookup_module(&self, port_id: &PortId) -> Option<ModuleId> {
        if port_id.as_str() == PORT_ID_STR {
            Some(ModuleId::new(MODULE_ID_STR.to_string()))
        } else {
            None
        }
    }
}
