#!/bin/bash

set -eou pipefail
cd $(dirname $0)

if [[ "${CAMERA:-}" == "" ]]; then
    printf "\x1b[38;5;203mError:\x1b[0m Must set CAMERA=...\n"
    exit 1
fi

# Setup.
export OUT=$CAMERA
source setup.sh

# Camera management.
fj repo add https://github.com/calder/fujixweekly
test camera_list    0 fj camera list
test recipe_load    0 fj recipe load -c4 pacific_blues
test recipe_save    0 fj recipe save -c4 test

# RAW conversion.
test convert        0 fj convert --out=tmp/out --recipe=pacific_blues ../images/$CAMERA/*.raf
open tmp/out/*.jpg
