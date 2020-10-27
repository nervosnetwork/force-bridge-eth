#!/bin/sh

if ! [ -f ckb.toml ]; then
  /bin/ckb init --chain "$CKB_CHAIN" --ba-arg "$BA_ARG" --ba-code-hash "$BA_CODE_HASH" --ba-hash-type "$BA_HASH_TYPE" --ba-message "$BA_MESSAGE"
fi

mkdir ckb_log \
	&& touch ckb_log/output.log ckb_log/error.log \
	&& ln -sf /dev/stdout ./ckb_log/output.log \
  && ln -sf /dev/stderr ./ckb_log/error.log

exec /bin/ckb run > ./ckb_log/output.log >> ./ckb_log/error.log &
sleep 3
exec /bin/ckb miner
