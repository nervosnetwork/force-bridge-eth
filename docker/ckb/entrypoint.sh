#!/bin/sh

if ! [ -f ckb.toml ]; then
<<<<<<< HEAD
  /bin/ckb init --chain "$CKB_CHAIN" --ba-arg "$BA_ARG" --ba-code-hash "$BA_CODE_HASH" --ba-hash-type "$BA_HASH_TYPE" --ba-message "$BA_MESSAGE"
  sed -i 's/filter = "info"/filter = "info,ckb-script=debug"/g' ckb.toml
=======
  /bin/ckb init --chain "$CKB_CHAIN" --ba-arg "$BA_ARG" --ba-code-hash "$BA_CODE_HASH" --ba-hash-type "$BA_HASH_TYPE" --ba-message "$BA_MESSAGE" \
  && sed -ic 's/filter = "info"/filter = "info,ckb-script=debug"/g' ckb.toml
>>>>>>> upstream/main
fi

exec /bin/ckb run &
sleep 3
exec /bin/ckb miner
