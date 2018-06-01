#!/bin/sh

set -ex

rm -f output/*
rm -f test
cargo build --manifest-path ../Cargo.toml
gcc -std=c11 -DDEBUG -o test test.c cjson.c -ansi -Wall -I../include -L../../target/debug -ldmbc_capi