#!/bin/bash

# Run the following commands as well if cargo setup is not installed
# curl https://sh.rustup.rs -sSf | sh
# source $HOME/.cargo/env

# run the following commands if searcher-examples is not already cloned
# git clone https://github.com/jito-labs/searcher-examples.git
# cd searcher-examples
# git checkout c6d9fd1e3644498db2b089149b65bf2057ef77d9
# git submodule update --init --recursive
# git apply ../print-change.patch
# cargo build --release
# cp ./target/release/jito-searcher-cli ~/
# cd ..
# sudo apt-get install -y libfontconfig1-dev

# install cargo if not installed, rust build system
if command -v cargo &> /dev/null
then
    echo "cargo is installed"
else
    echo "cargo is not installed"
    echo "installing cargo"
    curl https://sh.rustup.rs -sSf | sh
    source $HOME/.cargo/env
fi
# install solana CLI to talk using RPC
if command -v cargo &> /dev/null
then
    echo "Solana CLI is installed"
else
    echo "Solana CLI is not installed"
    echo "installing Solana CLI"
    sh -c "$(curl -sSfL https://release.solana.com/v1.17.3/install)"
fi
# dependency
sudo apt-get install -y libfontconfig1-dev
# set mainnet public RPC
solana config set --url https://api.mainnet-beta.solana.com
# build
cargo build --release

