<div align="center">
    <h1>sovereign-ibc</h1>
</div>

<div align="center">

[![Build Status][build-image]][build-link]
[![Apache 2.0 Licensed][license-image]][license-link]
![Rust 1.77+][rustc-image]

</div>

This repository contains the IBC implementation for the Sovereign SDK chains using `ibc-rs`.

## Build Guide

Please clone this repository with the included submodules to build using `cargo build`:

```sh
git clone --recurse-submodules <repo-addr>
```

If the repository was cloned without submodules, they can be fetched later:

```sh
git clone <repo-addr>
...
git submodule update --init
```

[//]: # (badges)
[build-image]: https://github.com/informalsystems/sovereign-ibc/workflows/Rust/badge.svg
[build-link]: https://github.com/informalsystems/sovereign-ibc/actions?query=workflow%3ARust
[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/informalsystems/sovereign-ibc/blob/main/LICENSE
[rustc-image]: https://img.shields.io/badge/rustc-1.77+-blue.svg