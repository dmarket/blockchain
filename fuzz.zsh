#!/bin/env zsh

mkdir -p fuzz-in
cargo afl build --all &&
./target/debug/setup-fuzz &&
if [[ ! -d fuzz-out ]]
then
	echo 'Starting new fuzz.'
	cargo afl fuzz -i fuzz-in -o fuzz-out -m 1024 -M fuzz-master ./target/debug/fuzz
else
	echo 'Continuing fuzz.'
	cargo afl fuzz -i - -o fuzz-out -m 1024 -M fuzz-master ./target/debug/fuzz
fi

