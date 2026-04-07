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
test recipe_load    0 fj recipe load -c1 pacific_blues
test recipe_save    0 fj recipe save -c1 test
