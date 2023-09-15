use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use ibc::applications::transfer::{MODULE_ID_STR, PORT_ID_STR};
use ibc::core::ics24_host::identifier::PortId;
use ibc::core::router::{self, ModuleId, Router};
use sov_ibc_transfer::context::TransferContext;
use sov_state::WorkingSet;

use crate::Ibc;

pub struct IbcRouter<'ws, 'c, C, Da>
where
    C: sov_modules_api::Context,
    Da: sov_modules_api::DaSpec,
{
    pub transfer_ctx: TransferContext<'ws, 'c, C>,
    _da: PhantomData<Da>,
}

impl<'ws, 'c, C, Da> IbcRouter<'ws, 'c, C, Da>
where
    C: sov_modules_api::Context,
    Da: sov_modules_api::DaSpec,
{
    pub fn new(
        ibc_mod: &Ibc<C, Da>,
        sdk_context: &'c C,
        working_set: Rc<RefCell<&'ws mut WorkingSet<C::Storage>>>,
    ) -> IbcRouter<'ws, 'c, C, Da> {
        IbcRouter {
            transfer_ctx: TransferContext::new(ibc_mod.transfer.clone(), sdk_context, working_set),
            _da: PhantomData,
        }
    }
}

impl<'ws, 'c, C, Da> Router for IbcRouter<'ws, 'c, C, Da>
where
    C: sov_modules_api::Context,
    Da: sov_modules_api::DaSpec,
{
    fn get_route(&self, module_id: &ModuleId) -> Option<&dyn router::Module> {
        if *module_id == ModuleId::new(MODULE_ID_STR.to_string()) {
            Some(&self.transfer_ctx)
        } else {
            None
        }
    }

    fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn router::Module> {
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
