#!/usr/bin/env bash

#sudo -s
# Exit script as soon as a command fails.
echo "" |  ./demo/vendor/ckb-cli account import --privkey-path demo/privkeys/ckb_key
echo "" |  ./demo/vendor/ckb-cli account import --privkey-path demo/privkeys/eth2ckb_relayer_key
echo "" |  ./demo/vendor/ckb-cli wallet transfer --capacity 9000000000 --to-address ckt1qyq2lc9zkdu9ha337xdrrf9lm2gtphdsweuqupv5r4  --from-account ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37 --tx-fee 0.1
sleep 20
./demo/vendor/ckb-cli wallet get-capacity --address ckt1qyq2lc9zkdu9ha337xdrrf9lm2gtphdsweuqupv5r4
