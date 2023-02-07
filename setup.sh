#!/bin/bash

# Run the following commands as well if cargo setup is not installed
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env

# run the following commands if searcher-examples is not already cloned
git clone https://github.com/jito-labs/searcher-examples.git
cd searcher-examples
git checkout c6d9fd1e3644498db2b089149b65bf2057ef77d9
git submodule update --init --recursive
git apply ../print-change.patch
cargo build --release
cp ./target/release/jito-searcher-cli ~/
cd ..

cargo build --release