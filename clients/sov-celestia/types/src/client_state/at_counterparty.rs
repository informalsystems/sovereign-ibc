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

#[cfg(test)]
mod test {
    use crate::client_state::SovTmClientState;

    use super::*;

    #[test]
    fn test_any_deserialization() {
        let value = Any {
            type_url: "/ibc.lightclients.wasm.v1.ClientState".into(),
            value: [
                10, 184, 1, 10, 53, 47, 105, 98, 99, 46, 108, 105, 103, 104, 116, 99, 108, 105,
                101, 110, 116, 115, 46, 115, 111, 118, 101, 114, 101, 105, 103, 110, 46, 116, 101,
                110, 100, 101, 114, 109, 105, 110, 116, 46, 118, 49, 46, 67, 108, 105, 101, 110,
                116, 83, 116, 97, 116, 101, 18, 127, 10, 98, 10, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 2, 16, 1, 26,
                34, 10, 32, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 34, 4, 8, 128, 244, 3, 42, 2, 16, 6, 58, 12, 115, 111, 118,
                95, 105, 98, 99, 47, 73, 98, 99, 47, 18, 25, 10, 7, 112, 114, 105, 118, 97, 116,
                101, 18, 4, 8, 1, 16, 3, 26, 4, 8, 128, 232, 7, 34, 2, 8, 3, 18, 32, 37, 136, 84,
                174, 68, 139, 38, 78, 69, 208, 181, 6, 183, 133, 171, 222, 47, 28, 47, 151, 25,
                151, 71, 171, 47, 166, 160, 15, 141, 74, 158, 220, 26, 2, 16, 6,
            ]
            .into(),
        };

        ClientStateAtCounterParty::<SovTmClientState>::try_from(value).unwrap();
    }
}
