#!/bin/bash

set -eou pipefail
cd $(dirname $0)

# Setup.
export OUT="HOOTL"
source setup.sh

# Basics.
test welcome        2 fj
test help           0 fj --help

# Recipe management.
test repo_add       0 fj repo add https://github.com/calder/fujixweekly
test repo_add2      0 fj repo add https://github.com/calder/fujinx-recipes
test repo_list      0 fj repo list
test recipe_list    0 fj recipe list
test repo_del2      0 fj repo del https://github.com/calder/fujinx-recipes
