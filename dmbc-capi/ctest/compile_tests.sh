#!/bin/sh

set -ex

rm -f test
cargo build
gcc -DDEBUG -o test test.c cjson.c -ansi -Wall -I../include -L../../target/debug -ldmbc_capi