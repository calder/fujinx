#!/bin/bash

if [[ "${OUT:-}" == "" ]]; then
    printf "\x1b[38;5;203mError:\x1b[0m Must set OUT=...\n"
    exit 1
fi

STEP=1
function test() {
    TEST=$(printf "out/$OUT/%02d_$1" $STEP); shift
    STATUS=$1; shift;
    echo "+ $@"
    status=0; $@ 1>$TEST.stdout 2>$TEST.stderr || status=$?
    if [[ $status != $STATUS ]]; then
        printf "\x1b[38;5;203mError:\x1b[0m Exited with status $status (expected $STATUS)\n"
        exit $status
    fi
    ((STEP++))
}

cargo install --locked --path ../..
rm -rf config "out/$OUT"; mkdir -p config "out/$OUT"
export FUJINX_CONFIG="$PWD/config"
