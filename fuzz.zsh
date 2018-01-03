#!/bin/env zsh

mkdir -p fuzz-in
cargo afl build &&
cargo afl run --bin setup-fuzz &&
if [[ ! -d fuzz-out ]]
then
	echo 'Starting new fuzz.'
	cargo afl fuzz -i fuzz-in -o fuzz-out -m 1024 ./target/debug/fuzz
else
	echo 'Continuing fuzz.'
	cargo afl fuzz -i - -o fuzz-out -m 1024 ./target/debug/fuzz
fi

