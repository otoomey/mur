#!/bin/bash

TEMPD=$(mktemp -d)

if [ ! -e "$TEMPD" ]; then
    >&2 echo "Failed to create temp directory"
    exit 1
fi

function compile () {
    if [[ $1 == *.c ]] 
    then
        FNAME=$(basename $1 .c)
        clang -O3 -nostdlib --target=riscv64 -nostdlib -march=rv64i -mabi=lp64 -mno-relax -S -o "$TEMPD/asm.s" "$1"
        clang -Wl,-Ttext=0x0 -nostdlib --target=riscv64 -march=rv64i -mabi=lp64 -mno-relax -o "$TEMPD/bin" "$TEMPD/asm.s"
    else
        clang -Wl,-Ttext=0x0 -nostdlib --target=riscv64 -march=rv64i -mabi=lp64 -mno-relax -o "$TEMPD/bin" "$1"
    fi
    FNAME=$(basename $1 .s)
    FNAME=$(basename $FNAME .c)
    llvm-objcopy -O binary "$TEMPD/bin" "./out/$FNAME.bin"
}

mkdir -p out
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
