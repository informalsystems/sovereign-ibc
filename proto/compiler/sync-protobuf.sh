#!/usr/bin/env bash

set -eou pipefail

# syn-protobuf.sh is a bash script to sync the protobuf files.
#
# This script should be run from the root directory of sovereign-ibc.
#
# This script will checkout the protobuf files from the git versions specified
# in proto/compiler/IBC_GO_COMMIT. If you want to sync the protobuf files to a
# newer version, modify the corresponding of those 2 files by specifying the
# commit ID that you wish to checkout from.

# We can specify where to clone the git repositories for ibc-go. By default they
# are cloned to proto/tmp/ibc-go.git. We can override this to existing
# directories that already have a clone of the repositories, so that there is no
# need to clone the entire repositories over and over again every time the
# script is called.

CACHE_PATH="${XDG_CACHE_HOME:-$HOME/.cache}"
IBC_GO_GIT="${IBC_GO_GIT:-$CACHE_PATH/ibc-go.git}"
IBC_GO_COMMIT="$(cat proto/compiler/IBC_GO_COMMIT)"
echo "IBC_GO_COMMIT: $IBC_GO_COMMIT"

# If the git directories does not exist, clone them as bare git repositories so
# that no local modification can be done there.
if [[ ! -e "$IBC_GO_GIT" ]]
then
    echo "Cloning ibc-go source code to as bare git repository to $IBC_GO_GIT"
    git clone --mirror https://github.com/cosmos/ibc-go.git "$IBC_GO_GIT"
else
    echo "Using existing ibc-go bare git repository at $IBC_GO_GIT"
fi

# Update the repositories using git fetch. This is so that we keep local copies
# of the repositories up to sync first.
pushd "$IBC_GO_GIT"
git fetch
popd

# Create a new temporary directory to check out the actual source files from the
# bare git repositories. This is so that we do not accidentally use an unclean
# local copy of the source files to generate the protobuf.
IBC_GO_DIR=$(mktemp -d /tmp/ibc-go-XXXXXXXX)

pushd "$IBC_GO_DIR"
git clone "$IBC_GO_GIT" .
git checkout -b "$IBC_GO_COMMIT" "$IBC_GO_COMMIT"

cd proto
buf mod update
buf export -v -o ../proto-include
popd

# Remove the existing generated protobuf files so that the newly generated code
# does not contain removed files.
rm -rf proto/src/prost
mkdir -p proto/src/prost

cd proto/compiler

# Build the compiler binary
cargo build

cargo run -- compile \
  --ibc "$IBC_GO_DIR/proto-include" \
  --out ./../src/prost

# Remove unused generated code
rm -f ./../src/prost/cosmos_proto.rs
rm -f ./../src/prost/cosmos.ics23.v1.rs
rm -f ./../src/prost/cosmos.upgrade.v1beta1.rs

# Remove the temporary checkouts of the repositories
rm -rf "$IBC_GO_DIR"
