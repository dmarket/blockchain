#!/usr/bin/env bash

export RUST_LOG=info
export CONFIG_PATH="./etc/config$1.toml"

cargo run -p dmbc-node