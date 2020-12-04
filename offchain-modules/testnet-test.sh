#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -o errexit
set -o xtrace
export RUST_BACKTRACE=1
export RUST_LOG=info,force=debug

bash ./deploy.sh
bash ./start-services.sh
bash ../demo/crosschain.sh
bash ./stop-services.sh
