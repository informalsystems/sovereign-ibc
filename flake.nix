{
  description = "Nix flake for building Wasm file";

  inputs = {
    nixpkgs.url = github:nixos/nixpkgs/nixpkgs-unstable;
    flake-utils.url = github:numtide/flake-utils;
    rust-overlay.url = github:oxalica/rust-overlay;

    sovereign-sdk-src = {
        flake = false;
        url = git+ssh://git@github.com/informalsystems/sovereign-sdk-wip?rev=356394cce29cbb9f56d4c32e2b5d0de1c1189ee8;
    };
  };

  outputs = inputs:
        let
            utils = inputs.flake-utils.lib;
        in
            utils.eachSystem
            [
                "aarch64-darwin"
                "aarch64-linux"
                "x86_64-darwin"
                "x86_64-linux"
            ]
        (system: let
            nixpkgs = import inputs.nixpkgs {
                inherit system;
                overlays = [
                    inputs.rust-overlay.overlays.default
                ];
            };

            rust-bin = nixpkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

            sov-celestia-cw = import ./nix/sov-celestia-cw.nix {
                inherit nixpkgs rust-bin;
                inherit (inputs) sovereign-sdk-src;
            };
        in {
            packages = {
                inherit (sov-celestia-cw) sov-celestia-cw;
            };
        });
}
