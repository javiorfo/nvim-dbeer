#!/usr/bin/env bash

ROOT=$1

(cd $ROOT/rust && cargo build --release)

if [ $? -ne 0 ]; then
    exit 1
fi

BIN=$ROOT/bin
mkdir -p $BIN

mv $ROOT/rust/target/release/dbeer $BIN

(cd $ROOT/rust && cargo clean)
