{
    nixpkgs
,   rust-bin
,   sovereign-sdk-src
}:
let
    sov-celestia-src = nixpkgs.stdenv.mkDerivation {
        name = "sov-celestia-src";
        dontUnpack = true;
        dontBuild = true;

        installPhase = ''
            mkdir -p $out $out/vendor

            cp -r ${../clients} $out/clients
            cp -r ${../modules} $out/modules
            cp -r ${../mocks} $out/mocks
            cp -r ${../proto} $out/proto
            cp -r ${sovereign-sdk-src} $out/vendor/sovereign-sdk
            cp ${../Cargo.lock} $out/Cargo.lock

            cat ${../Cargo.toml} > $out/Cargo.toml
            cat ${../.cargo/config.toml} >> $out/Cargo.toml
        '';
    };

    sov-celestia-cw = nixpkgs.rustPlatform.buildRustPackage {
        name = "sov-celestia-cw";
        src = sov-celestia-src;

        cargoLock = {
            lockFile = ../Cargo.lock;
            outputHashes = {
                "basecoin-0.1.0" = "sha256-NCjfbrtxo7NZ/pW2FxAf+bB9kmluQLuI0zekwX2lypY=";
                "celestia-proto-0.1.0" = "sha256-iUgrctxdJUyhfrEQ0zoVj5AKIqgj/jQVNli5/K2nxK0=";
                "ibc-0.51.0" = "sha256-6HcMSTcKx4y1hfKcXnzgiNM9FWdqUuUNnemAgS36Z1A=";
                "jmt-0.9.0" = "sha256-pq1v6FXS//6Dh+fdysQIVp+RVLHdXrW5aDx3263O1rs=";
                "nmt-rs-0.1.0" = "sha256-jcHbqyIKk8ZDDjSz+ot5YDxROOnrpM4TRmNFVfNniwU=";
                "risc0-cycle-utils-0.3.0" = "sha256-5dA62v1eqfyZBny4s3YlC2Tty7Yfd/OAVGfTlLBgypk=";
                "rockbound-1.0.0" = "sha256-xTaeBndRb/bYe+tySChDKsh4f9pywAExsdgJExCQiy8=";
                "tendermint-0.32.0" = "sha256-FtY7a+hBvQryATrs3mykCWFRe8ABTT6cuf5oh9IBElQ=";
            };
        };

        nativeBuildInputs = [
            rust-bin
        ];

        doCheck = false;

        CONSTANTS_MANIFEST = sov-celestia-src;

        buildPhase = ''
            RUSTFLAGS='-C link-arg=-s' cargo build -p sov-celestia-client-cw --target wasm32-unknown-unknown --release --lib --locked
        '';

        installPhase = ''
            mkdir -p $out
            cp target/wasm32-unknown-unknown/release/sov_celestia_client_cw.wasm $out/
        '';
    };
in
{
    inherit sov-celestia-cw;
}