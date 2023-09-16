#!/bin/bash

TEMPD=$(mktemp -d)

if [ ! -e "$TEMPD" ]; then
    >&2 echo "Failed to create temp directory"
    exit 1
fi

function compile () {
    clang -Wl,-Ttext=0x0 -nostdlib --target=riscv64 -march=rv64g -mno-relax -o "$TEMPD/bin" "$1"
    FNAME=$(basename $1 .s)
    llvm-objcopy -O binary "$TEMPD/bin" "./out/$FNAME.bin"
}

if [[ -d $1 ]]; then
    for entry in "$1"/*
    do
        compile "$entry"
    done
elif [[ -f $1 ]]; then
    compile $1
fi

trap "exit 1"           HUP INT PIPE QUIT TERM
trap 'rm -rf "$TEMPD"'  EXIT
