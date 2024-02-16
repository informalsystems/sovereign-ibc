use std::str::FromStr;

use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, Order, Storage};
use ibc_client_wasm_types::client_state::ClientState as WasmClientState;
use ibc_core::client::context::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation,
};
use ibc_core::client::context::consensus_state::ConsensusState as _;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::{Height, UpdateKind};
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::{
    iteration_key, ClientConsensusStatePath, ClientUpdateHeightPath, ClientUpdateTimePath,
    ITERATE_CONSENSUS_STATE_PREFIX,
};
use ibc_core::host::ValidationContext;
use ibc_proto::google::protobuf::Any;
use prost::Message;
use sov_celestia_client::client_state::ClientState;
use sov_celestia_client::types::client_message::ClientMessage;

use crate::types::{
    parse_height, CheckForMisbehaviourMsg, ContractError, ContractResult, ExecuteMsg,
    ExportMetadataMsg, GenesisMetadata, HeightTravel, InstantiateMsg, QueryMsg, QueryResponse,
    StatusMsg, UpdateStateMsg, UpdateStateOnMisbehaviourMsg, VerifyClientMessageMsg,
    VerifyMembershipMsg, VerifyNonMembershipMsg, VerifyUpgradeAndUpdateStateMsg,
};

pub type Checksum = Vec<u8>;

/// Context is a wrapper around the deps and env that gives access to the
/// methods of the ibc-rs Validation and Execution traits.
pub struct Context<'a> {
    deps: Option<Deps<'a>>,
    deps_mut: Option<DepsMut<'a>>,
    env: Env,
    client_id: ClientId,
    checksum: Option<Checksum>,
}

impl<'a> Context<'a> {
    pub fn new_ref(deps: Deps<'a>, env: Env) -> Result<Self, ContractError> {
        let client_id = ClientId::from_str(env.contract.address.as_str())?;

        Ok(Self {
            deps: Some(deps),
            deps_mut: None,
            env,
            client_id,
            checksum: None,
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

    pub fn retrieve(&self, key: impl AsRef<[u8]>) -> Result<Vec<u8>, ClientError> {
        let value = self
            .storage_ref()
            .get(key.as_ref())
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
            self.client_id().clone(),
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

    pub fn encode_client_state(&self, client_state: ClientState) -> Result<Vec<u8>, ClientError> {
        let wasm_client_state = WasmClientState {
            data: Any::from(client_state.clone()).encode_to_vec(),
            checksum: self.checksum.clone().ok_or(ClientError::Other {
                description: "checksum not set".to_string(),
            })?,
            latest_height: client_state.latest_height(),
        };

        Ok(Any::from(wasm_client_state).encode_to_vec())
    }

    pub fn instantiate(&mut self, msg: InstantiateMsg) -> Result<Binary, ContractError> {
        let any = Any::decode(&mut msg.client_state.as_slice())?;

        let client_state = ClientState::try_from(any)?;

        let any_consensus_state = Any::decode(&mut msg.consensus_state.as_slice())?;

        self.set_checksum(msg.checksum);

        client_state.initialise(self, &self.client_id(), any_consensus_state)?;

        Ok(to_json_binary(&ContractResult::success())?)
    }

    pub fn execute(&mut self, msg: ExecuteMsg) -> Result<Binary, ContractError> {
        let client_id = self.client_id();

        let client_state = self.client_state(&client_id)?;

        let result = match msg {
            ExecuteMsg::VerifyMembership(msg) => {
                let msg = VerifyMembershipMsg::try_from(msg)?;

                let client_cons_state_path = ClientConsensusStatePath::new(
                    self.client_id(),
                    msg.height.revision_number(),
                    msg.height.revision_height(),
                );

                let consensus_state = self.consensus_state(&client_cons_state_path)?;

                client_state.verify_membership(
                    &msg.prefix,
                    &msg.proof,
                    consensus_state.root(),
                    msg.path,
                    msg.value,
                )?;

                ContractResult::success()
            }
            ExecuteMsg::VerifyNonMembership(msg) => {
                let msg = VerifyNonMembershipMsg::try_from(msg)?;

                let client_cons_state_path = ClientConsensusStatePath::new(
                    client_id.clone(),
                    msg.height.revision_number(),
                    msg.height.revision_height(),
                );

                let consensus_state = self.consensus_state(&client_cons_state_path)?;

                client_state.verify_non_membership(
                    &msg.prefix,
                    &msg.proof,
                    consensus_state.root(),
                    msg.path,
                )?;

                ContractResult::success()
            }
            ExecuteMsg::VerifyClientMessage(msg) => {
                let msg = VerifyClientMessageMsg::try_from(msg)?;

                let any_client_msg: Any = match msg.client_message {
                    ClientMessage::Header(header) => (*header).into(),
                    ClientMessage::Misbehaviour(misbehaviour) => (*misbehaviour).into(),
                };

                client_state.verify_client_message(self, &client_id, any_client_msg)?;

                ContractResult::success()
            }
            ExecuteMsg::CheckForMisbehaviour(msg) => {
                let msg = CheckForMisbehaviourMsg::try_from(msg)?;

                let any_client_msg: Any = match msg.client_message {
                    ClientMessage::Header(header) => (*header).into(),
                    ClientMessage::Misbehaviour(misbehaviour) => (*misbehaviour).into(),
                };

                let result =
                    client_state.check_for_misbehaviour(self, &client_id, any_client_msg)?;

                ContractResult::success().misbehaviour(result)
            }
            ExecuteMsg::UpdateStateOnMisbehaviour(msg_raw) => {
                let msg: UpdateStateOnMisbehaviourMsg =
                    UpdateStateOnMisbehaviourMsg::try_from(msg_raw)?;

                let any_client_msg = match msg.client_message {
                    ClientMessage::Header(header) => (*header).into(),
                    ClientMessage::Misbehaviour(misbehaviour) => (*misbehaviour).into(),
                };

                client_state.update_state_on_misbehaviour(self, &client_id, any_client_msg)?;

                ContractResult::success()
            }
            ExecuteMsg::UpdateState(msg_raw) => {
                let msg = UpdateStateMsg::try_from(msg_raw)?;

                let (_, any_client_msg) = match msg.client_message {
                    ClientMessage::Header(header) => (UpdateKind::UpdateClient, (*header).into()),
                    ClientMessage::Misbehaviour(misbehaviour) => {
                        (UpdateKind::SubmitMisbehaviour, (*misbehaviour).into())
                    }
                };

                client_state.update_state(self, &client_id, any_client_msg)?;

                ContractResult::success()
            }
            ExecuteMsg::CheckSubstituteAndUpdateState(_) => {
                ContractResult::error("ibc-rs does no support this feature yet".to_string())
            }
            ExecuteMsg::VerifyUpgradeAndUpdateState(msg) => {
                let msg = VerifyUpgradeAndUpdateStateMsg::try_from(msg)?;

                let client_cons_state_path = ClientConsensusStatePath::new(
                    client_id.clone(),
                    client_state.latest_height().revision_number(),
                    client_state.latest_height().revision_height(),
                );

                let consensus_state = self.consensus_state(&client_cons_state_path)?;

                client_state.verify_upgrade_client(
                    msg.upgrade_client_state.clone(),
                    msg.upgrade_consensus_state.clone(),
                    msg.proof_upgrade_client,
                    msg.proof_upgrade_consensus_state,
                    consensus_state.root(),
                )?;

                client_state.update_state_on_upgrade(
                    self,
                    &client_id,
                    msg.upgrade_client_state,
                    msg.upgrade_consensus_state,
                )?;

                ContractResult::success()
            }
        };
        Ok(to_json_binary(&result)?)
    }

    pub fn query(&self, msg: QueryMsg) -> Result<Binary, ContractError> {
        let client_id = self.client_id();

        let resp = match msg {
            QueryMsg::ClientTypeMsg(_) => unimplemented!("ClientTypeMsg"),
            QueryMsg::GetLatestHeightsMsg(_) => unimplemented!("GetLatestHeightsMsg"),
            QueryMsg::ExportMetadata(ExportMetadataMsg {}) => {
                QueryResponse::genesis_metadata(self.get_metadata()?)
            }
            QueryMsg::Status(StatusMsg {}) => {
                let client_state = self.client_state(&client_id)?;

                match client_state.status(self, &client_id) {
                    Ok(status) => QueryResponse::status(status.to_string()),
                    Err(err) => QueryResponse::status(err.to_string()),
                }
            }
        };

        Ok(to_json_binary(&resp)?)
    }
}

pub trait StorageRef {
    fn storage_ref(&self) -> &dyn Storage;
}

impl StorageRef for Context<'_> {
    fn storage_ref(&self) -> &dyn Storage {
        match self.deps {
            Some(ref deps) => deps.storage,
            None => panic!("storage should be available"),
        }
    }
}

pub trait StorageMut: StorageRef {
    fn storage_mut(&mut self) -> &mut dyn Storage;
}

impl StorageMut for Context<'_> {
    fn storage_mut(&mut self) -> &mut dyn Storage {
        match self.deps_mut {
            Some(ref mut deps) => deps.storage,
            None => panic!("storage should be available"),
        }
    }
}
