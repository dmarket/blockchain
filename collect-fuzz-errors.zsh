#!/bin/env zsh

for input in ./fuzz-out/crashes/*
do
    echo $input
    cargo afl run --bin fuzz -- < $input
    echo
done

