#!/bin/env zsh

mkdir -p fuzz-in
cargo afl build &&
cargo afl run --bin setup-fuzz &&
if [[ ! -d fuzz-out ]]
then
	echo 'Starting new fuzz.'
	urxvt -e cargo afl fuzz -i fuzz-in -o fuzz-out -m 1024 -M fuzz-master ./target/debug/fuzz &
	urxvt -e cargo afl fuzz -i fuzz-in -o fuzz-out -m 1024 -S fuzz-slave ./target/debug/fuzz &
else
	echo 'Continuing fuzz.'
	urxvt -e cargo afl fuzz -i - -o fuzz-out -m 1024 -M fuzz-master ./target/debug/fuzz &
	urxvt -e cargo afl fuzz -i - -o fuzz-out -m 1024 -S fuzz-slave ./target/debug/fuzz &
fi

