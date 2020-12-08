#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -o errexit
set -o xtrace
export RUST_BACKTRACE=1
export RUST_LOG=info,force=debug

PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"
cd "${PROJECT_DIR}"/offchain-modules

bash ./deploy.sh
bash ./start-services.sh
bash ../demo/crosschain.sh
bash ./stop-services.sh
