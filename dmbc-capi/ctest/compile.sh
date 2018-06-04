#!/bin/sh

set -ex

rm -f test
cargo build --manifest-path ../Cargo.toml
gcc -DDEBUG -o test test.c cjson.c -ansi -Wall -I../include -L../../target/debug -ldmbc_capi