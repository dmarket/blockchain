#!/bin/env zsh

mkdir -p fuzz-in
cargo afl build
cargo afl run --bin setup-fuzz
cargo afl fuzz -i fuzz-in -o fuzz-out -m 1024 ./target/debug/fuzz
