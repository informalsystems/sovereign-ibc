use cosmwasm_std::{Deps, DepsMut, Env, Storage};

/// Context is a wrapper around the deps and env that gives access to the
/// methods of the ibc-rs Validation and Execution traits.
pub struct ContextRef<'a> {
    pub deps: Deps<'a>,
    pub env: Env,
}

impl<'a> ContextRef<'a> {
    pub fn new(deps: Deps<'a>, env: Env) -> Self {
        Self { deps, env }
    }

    pub fn log(&self, msg: &str) {
        self.deps.api.debug(msg)
    }
}

pub trait StorageRef {
    fn storage(&self) -> &dyn Storage;
}

impl StorageRef for ContextRef<'_> {
    fn storage(&self) -> &dyn Storage {
        self.deps.storage
    }
}

pub struct ContextMut<'a> {
    pub deps: DepsMut<'a>,
    pub env: Env,
}

impl<'a> ContextMut<'a> {
    pub fn new(deps: DepsMut<'a>, env: Env) -> Self {
        Self { deps, env }
    }

    pub fn log(&self, msg: &str) {
        self.deps.api.debug(msg)
    }
}

pub trait StorageMut: StorageRef {
    fn storage_mut(&mut self) -> &mut dyn Storage;
}

impl StorageRef for ContextMut<'_> {
    fn storage(&self) -> &dyn Storage {
        self.deps.storage
    }
}

impl StorageMut for ContextMut<'_> {
    fn storage_mut(&mut self) -> &mut dyn Storage {
        self.deps.storage
    }
}
