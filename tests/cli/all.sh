#!/bin/bash

set -eou pipefail
cd $(dirname $0)

./hootl.sh
./hitl.sh
