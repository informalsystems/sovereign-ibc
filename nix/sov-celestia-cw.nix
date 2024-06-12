{
    nixpkgs
,   rust-bin
,   sovereign-sdk-src
}:
let
    # Use local file path instead of git URL for sovereign-sdk dependencies
    patch-section = builtins.replaceStrings
        [
            "# path = \"vendor/sovereign-sdk"
            "git = \"ssh://git@github.com/informalsystems/sovereign-sdk-wip.git\"\nrev = "
        ]
        [
            "path = \"vendor/sovereign-sdk"
            "# git = \"ssh://git@github.com/informalsystems/sovereign-sdk-wip.git\"\n# rev = "
        ]
        (builtins.readFile ../.cargo/config.toml)
    ;

    # Comment out Cargo.lock lines that include sovereign-sdk source, since we switch to local file path
    cargo-lock = builtins.replaceStrings
        [ "source = \"git+ssh://git@github.com/informalsystems/sovereign-sdk-wip.git" ]
        [ "# source = \"git+ssh://git@github.com/informalsystems/sovereign-sdk-wip.git" ]
        (builtins.readFile ../Cargo.lock)
    ;

    cargo-toml-file = builtins.toFile
        "Cargo.toml"
        ( builtins.concatStringsSep
            "\n"
            [
                (builtins.readFile ../Cargo.toml)
                patch-section
            ]
        )
    ;

    cargo-lock-file = builtins.toFile
        "Cargo.lock"
        cargo-lock
    ;

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

            cp ${cargo-lock-file} $out/Cargo.lock
            cp ${cargo-toml-file} $out/Cargo.toml
        '';
    };

    sov-celestia-cw = nixpkgs.rustPlatform.buildRustPackage {
        name = "sov-celestia-cw";
        src = sov-celestia-src;

        cargoLock = {
            lockFile = cargo-lock-file;
            outputHashes = {
                "basecoin-0.1.0" = "sha256-CY1U6z18oAv9iFDXeS5YNgK3cOMGrkChJSJ2iMFLXvg=";
                "celestia-proto-0.1.0" = "sha256-iUgrctxdJUyhfrEQ0zoVj5AKIqgj/jQVNli5/K2nxK0=";
                "ibc-0.53.0" = "sha256-sKgQB+tKKnIBDqBCf+OdRv72JCdFBB3r7LKNLLQVQS0=";
                "jmt-0.9.0" = "sha256-pq1v6FXS//6Dh+fdysQIVp+RVLHdXrW5aDx3263O1rs=";
                "nmt-rs-0.1.0" = "sha256-jcHbqyIKk8ZDDjSz+ot5YDxROOnrpM4TRmNFVfNniwU=";
                "risc0-cycle-utils-0.3.0" = "sha256-5dA62v1eqfyZBny4s3YlC2Tty7Yfd/OAVGfTlLBgypk=";
                "rockbound-1.0.0" = "sha256-aDrNegRfsSwiNw4XLsE4rpUYDZn2N59UJbGZ6mpY180=";
                "tendermint-0.32.0" = "sha256-FtY7a+hBvQryATrs3mykCWFRe8ABTT6cuf5oh9IBElQ=";
                "cosmos-sdk-proto-0.22.0-pre" = "sha256-r7zoHmk12iTdsyJ/4rRgD5HDeDSiNcs/HlS7eGi+NbE=";
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