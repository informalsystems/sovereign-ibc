# sov-ibc-proto

sov-ibc-proto encompasses essential protobuf definitions, a compiler, and
Rust-generated data structures crucial for IBC consumers, including IBC
relayers, modules, or light clients. This directory comprises the following
components:

- **definitions**: Contains .proto files for both Sovereign SDK rollups and
  light clients, providing the foundational protobuf definitions.

- **compiler**: Includes a binary used by the `sync-protobuf.sh` script. This
  binary is utilized to generate the Rust representation of the protobuf
  definitions.

- **Library**: The `sov-ibc-proto` crate encapsulates the Protobuf-generated
data structures required for interacting either with IBC modules or light
clients.

## Code Generation

To generate Rust code or sync Protobuf files, run the `sync-protobuf.sh` script
from the root directory of the `sovereign-ibc` repository:

```bash
./proto/compiler/sync-protobuf.sh
```
