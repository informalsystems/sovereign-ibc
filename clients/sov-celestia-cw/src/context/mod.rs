pub mod client_ctx;
pub mod custom_ctx;

use std::str::FromStr;

use cosmwasm_std::{Deps, DepsMut, Env, Order, Storage};
use ibc_client_wasm_types::client_state::ClientState as WasmClientState;
use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::{
    iteration_key, ClientStatePath, ClientUpdateHeightPath, ClientUpdateTimePath,
    ITERATE_CONSENSUS_STATE_PREFIX,
};
use ibc_core::primitives::proto::{Any, Protobuf};
use prost::Message;

use crate::types::{ClientType, ContractError, GenesisMetadata, HeightTravel, MigrationPrefix};
use crate::utils::{parse_height, AnyCodec};

type Checksum = Vec<u8>;

/// Context is a wrapper around the deps and env that gives access to the
/// methods under the ibc-rs Validation and Execution traits.
pub struct Context<'a, C: ClientType<'a>> {
    deps: Option<Deps<'a>>,
    deps_mut: Option<DepsMut<'a>>,
    env: Env,
    client_id: ClientId,
    checksum: Option<Checksum>,
    migration_prefix: MigrationPrefix,
    client_type: std::marker::PhantomData<C>,
}

impl<'a, C: ClientType<'a>> Context<'a, C> {
    pub fn new_ref(deps: Deps<'a>, env: Env) -> Result<Self, ContractError> {
        let client_id = ClientId::from_str(env.contract.address.as_str())?;

        Ok(Self {
            deps: Some(deps),
            deps_mut: None,
            env,
            client_id,
            checksum: None,
            migration_prefix: MigrationPrefix::None,
            client_type: std::marker::PhantomData::<C>,
        })
    }

    pub fn new_mut(deps: DepsMut<'a>, env: Env) -> Result<Self, ContractError> {
        let client_id = ClientId::from_str(env.contract.address.as_str())?;

        Ok(Self {
            deps: None,
            deps_mut: Some(deps),
            env,
            client_id,
            checksum: None,
            migration_prefix: MigrationPrefix::None,
            client_type: std::marker::PhantomData::<C>,
        })
    }

    pub fn env(&self) -> &Env {
        &self.env
    }

    pub fn log(&self, msg: &str) -> Option<()> {
        self.deps.map(|deps| deps.api.debug(msg))
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id.clone()
    }

    pub fn set_checksum(&mut self, checksum: Checksum) {
        self.checksum = Some(checksum);
    }

    pub fn set_subject_prefix(&mut self) {
        self.migration_prefix = MigrationPrefix::Subject;
    }

    pub fn set_substitute_prefix(&mut self) {
        self.migration_prefix = MigrationPrefix::Substitute;
    }

    pub fn prefixed_key(&self, key: impl AsRef<[u8]>) -> Vec<u8> {
        let mut prefixed_key = Vec::new();
        prefixed_key.extend_from_slice(self.migration_prefix.key());
        prefixed_key.extend_from_slice(key.as_ref());

        prefixed_key
    }

    pub fn retrieve(&self, key: impl AsRef<[u8]>) -> Result<Vec<u8>, ClientError> {
        let prefixed_key = self.prefixed_key(key);

        let value = self
            .storage_ref()
            .get(prefixed_key.as_ref())
            .ok_or(ClientError::Other {
                description: "key not found".to_string(),
            })?;

        Ok(value)
    }

    pub fn insert(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
        self.storage_mut().set(key.as_ref(), value.as_ref());
    }

    pub fn remove(&mut self, key: impl AsRef<[u8]>) {
        self.storage_mut().remove(key.as_ref());
    }

    pub fn get_heights(&self) -> Result<Vec<Height>, ClientError> {
        let iterator = self.storage_ref().range(None, None, Order::Ascending);

        iterator.map(|(_, height)| parse_height(height)).collect()
    }

    /// Searches for either the earliest next or latest previous height based on
    /// the given height and travel direction.
    pub fn get_adjacent_height(
        &self,
        height: &Height,
        travel: HeightTravel,
    ) -> Result<Option<Height>, ClientError> {
        let iteration_key = iteration_key(height.revision_number(), height.revision_height());

        let mut iterator = match travel {
            HeightTravel::Prev => {
                self.storage_ref()
                    .range(None, Some(&iteration_key), Order::Descending)
            }
            HeightTravel::Next => {
                self.storage_ref()
                    .range(Some(&iteration_key), None, Order::Ascending)
            }
        };

        iterator
            .next()
            .map(|(_, height)| parse_height(height))
            .transpose()
    }

    pub fn client_update_time_key(&self, height: &Height) -> Vec<u8> {
        let client_update_time_path = ClientUpdateTimePath::new(
            self.client_id(),
            height.revision_number(),
            height.revision_height(),
        );

        client_update_time_path.leaf().into_bytes()
    }

    pub fn client_update_height_key(&self, height: &Height) -> Vec<u8> {
        let client_update_height_path = ClientUpdateHeightPath::new(
            self.client_id(),
            height.revision_number(),
            height.revision_height(),
        );

        client_update_height_path.leaf().into_bytes()
    }

    pub fn get_metadata(&self) -> Result<Option<Vec<GenesisMetadata>>, ContractError> {
        let mut metadata = Vec::<GenesisMetadata>::new();

        let start_key = ITERATE_CONSENSUS_STATE_PREFIX.to_string().into_bytes();

        let iterator = self
            .storage_ref()
            .range(Some(&start_key), None, Order::Ascending);

        for (_, encoded_height) in iterator {
            let height = parse_height(encoded_height);

            match height {
                Ok(height) => {
                    let processed_height_key = self.client_update_height_key(&height);
                    metadata.push(GenesisMetadata {
                        key: processed_height_key.clone(),
                        value: self.retrieve(&processed_height_key)?,
                    });
                    let processed_time_key = self.client_update_time_key(&height);
                    metadata.push(GenesisMetadata {
                        key: processed_time_key.clone(),
                        value: self.retrieve(&processed_time_key)?,
                    });
                }
                Err(_) => break,
            }
        }

        let iterator = self
            .storage_ref()
            .range(Some(&start_key), None, Order::Ascending);

        for (key, height) in iterator {
            metadata.push(GenesisMetadata { key, value: height });
        }

        Ok(Some(metadata))
    }

    pub fn obtain_checksum(&self) -> Result<Checksum, ClientError> {
        match &self.checksum {
            Some(checksum) => Ok(checksum.clone()),
            None => {
                let client_state_value = self.retrieve(ClientStatePath::leaf())?;

                let wasm_client_state: WasmClientState =
                    Protobuf::<Any>::decode(client_state_value.as_slice()).map_err(|e| {
                        ClientError::Other {
                            description: e.to_string(),
                        }
                    })?;

                Ok(wasm_client_state.checksum)
            }
        }
    }

    pub fn encode_client_state(
        &self,
        client_state: C::ClientState,
    ) -> Result<Vec<u8>, ClientError> {
        let wasm_client_state = WasmClientState {
            data: C::ClientState::encode_thru_any(client_state.clone()),
            checksum: self.obtain_checksum()?,
            latest_height: client_state.latest_height(),
        };

        Ok(Any::from(wasm_client_state).encode_to_vec())
    }
}

pub trait StorageRef {
    fn storage_ref(&self) -> &dyn Storage;
}

impl<'a, C: ClientType<'a>> StorageRef for Context<'a, C> {
    fn storage_ref(&self) -> &dyn Storage {
        match self.deps {
            Some(ref deps) => deps.storage,
            None => match self.deps_mut {
                Some(ref deps) => deps.storage,
                None => panic!("storage should be available"),
            },
        }
    }
}

pub trait StorageMut: StorageRef {
    fn storage_mut(&mut self) -> &mut dyn Storage;
}

impl<'a, C: ClientType<'a>> StorageMut for Context<'a, C> {
    fn storage_mut(&mut self) -> &mut dyn Storage {
        match self.deps_mut {
            Some(ref mut deps) => deps.storage,
            None => panic!("storage should be available"),
        }
    }
}
