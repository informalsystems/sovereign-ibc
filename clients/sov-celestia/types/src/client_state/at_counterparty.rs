use ibc_client_wasm_types::client_state::ClientState as WasmRawClientState;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::primitives::proto::Any;
use prost::Message;

use crate::sovereign::Error;

pub struct WasmClientState<CS> {
    pub client_state: CS,
    pub checksum: Vec<u8>,
    pub latest_height: Height,
}

impl<CS> TryFrom<WasmRawClientState> for WasmClientState<CS>
where
    CS: TryFrom<Any>,
    <CS as TryFrom<Any>>::Error: Into<ClientError>,
{
    type Error = Error;

    fn try_from(raw: WasmRawClientState) -> Result<Self, Self::Error> {
        let WasmRawClientState {
            data,
            checksum,
            latest_height,
        } = raw;

        let any_cs = Any::decode(data.as_slice()).map_err(Error::source)?;

        Ok(Self {
            client_state: any_cs.try_into().map_err(Into::into)?,
            checksum,
            latest_height,
        })
    }
}

pub enum ClientStateAtCounterParty<CS> {
    Native(CS),
    Wasm(WasmClientState<CS>),
}

impl<CS> ClientStateAtCounterParty<CS> {
    pub fn into_inner(self) -> CS {
        match self {
            Self::Native(client_state) | Self::Wasm(WasmClientState { client_state, .. }) => {
                client_state
            }
        }
    }
}

impl<CS> TryFrom<Any> for ClientStateAtCounterParty<CS>
where
    CS: TryFrom<Any>,
    <CS as TryFrom<Any>>::Error: Into<ClientError>,
{
    type Error = Error;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        WasmRawClientState::try_from(raw.clone())
            .map_err(Error::source)
            .and_then(WasmClientState::<CS>::try_from)
            .map(ClientStateAtCounterParty::Wasm)
            .or_else(|_| {
                Ok(CS::try_from(raw)
                    .map(ClientStateAtCounterParty::Native)
                    .map_err(Into::into)?)
            })
    }
}
