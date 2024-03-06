use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use ibc_app_transfer::types::{MODULE_ID_STR, PORT_ID_STR};
use ibc_core::host::types::identifiers::PortId;
use ibc_core::router::module::Module;
use ibc_core::router::router::Router;
use ibc_core::router::types::module::ModuleId;
use sov_ibc_transfer::context::IbcTransferContext;
use sov_modules_api::{Context, DaSpec, Spec, WorkingSet};

use crate::Ibc;

pub struct IbcRouter<'ws, S: Spec, Da: DaSpec> {
    pub transfer_ctx: IbcTransferContext<'ws, S>,
    _da: PhantomData<Da>,
}

impl<'ws, S: Spec, Da: DaSpec> IbcRouter<'ws, S, Da> {
    pub fn new(
        ibc_mod: &Ibc<S, Da>,
        sdk_context: Context<S>,
        working_set: Rc<RefCell<&'ws mut WorkingSet<S>>>,
    ) -> IbcRouter<'ws, S, Da> {
        IbcRouter {
            transfer_ctx: IbcTransferContext::new(
                ibc_mod.transfer.clone(),
                sdk_context,
                working_set,
            ),
            _da: PhantomData,
        }
    }
}

impl<'ws, S: Spec, Da: DaSpec> Router for IbcRouter<'ws, S, Da> {
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
