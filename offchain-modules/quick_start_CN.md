## 环境准备

### 运行 ETH 和 CKB节点

[docker-compose 运行](https://github.com/nervosnetwork/force-bridge-eth/tree/main/docker) 即可 ，会运行 `ckb`,   `ckb-indexer`,  `eth`  三个服务


### 账户说明

| 私钥                                                         | 地址                                             | 用途说明                               | 修改方式                                                        |
| ------------------------------------------------------------ | ------------------------------------------------ | ---------------------------------- | ------------------------------------------------------------ |
| `0xc4ad657963930fbff2e9de3404b30a4e21432c89952ed430b56bf802945ed37a` | `0x17c4b5CE0605F63732bfd175feCe7aC6b4620FD2`     | 执行 `lock unlock` 等`eth`合约操作  | 在 eth 的配置文件  [genesis.json](https://github.com/nervosnetwork/force-bridge-eth/blob/dedf9f05ccbfb606633e074d5f8789b5e9eb74be/docker/geth/geth-genesis.json#L14) 中修改 |
| `0x719e94ec5d2ecef67b5878503ffd6e1e0e2fe7a52ddd55c436878cb4d52d376d` | `0x8951a3DdEf2bB36fF3846C3B6968812C269f4561`     | `relay_ckb_header`                 | 在 eth 的配置文件  [genesis.json](https://github.com/nervosnetwork/force-bridge-eth/blob/dedf9f05ccbfb606633e074d5f8789b5e9eb74be/docker/geth/geth-genesis.json#L14) 中修改 |
| `0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc` | `ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37` | 执行 `mint burn`  等`ckb` 合约操作 | 默认生成并保存在 spec/dev.toml                               |


#### 合约部署

1、部署 ETH 合约,

- 需要提前准备好 `recipient-typescript code_hash` 

  ```toml
  code_hash = "94ad18d0329e195bb0a49c2f46ceb93ab50884331ad55f6df6ed012c1cc2d3bd"
  ```

  执行命令和结果：`hardhat.config.js` 中的私钥地址 就是后续 跨链操作的地址（部署合约才有 erc20）

  ```shell
   npx hardhat run scripts/geth/deployAll.js --network geth
  ```

  ```
  {
    "ERC20Deploy": "0xE9837e999934E2f61D4148CeEEb1Be3cc00Cb37c",
    "CKBChainDeploy": "0x3C049b35392D20e5C1E7bB6B32E26f07093A6923",
    "EaglesongDeploy": "0x4414f183026502850203cE9df08f2648E1C39b32",
    "TokenLockerDepoly": "0x30F58CEEEC7Ace3fC2f938F13dD2C0c69B4A19b0"
  }
  ```



将合约地址放入 环境变量，方便后续执行

```shell
export ERC20DeployAddr=0xE9837e999934E2f61D4148CeEEb1Be3cc00Cb37c
export CKBChainDeployAddr=0x3C049b35392D20e5C1E7bB6B32E26f07093A6923
export TokenLockerDepolyAddr=0x30F58CEEEC7Ace3fC2f938F13dD2C0c69B4A19b0
```



2、部署  CKB 合约 

```shell
RUST_LOG=info ./target/debug/force-eth-cli dev-init --eth-contract-address $TokenLockerDepolyAddr --eth-token-address $ERC20DeployAddr --private-key-path cli/privkeys/ckb_key -f
```

```
DevInitArgs {
    force: true,
    config_path: "/tmp/.force-bridge-cli/config.toml",
    rpc_url: "http://127.0.0.1:8114",
    indexer_url: "http://127.0.0.1:8116",
    private_key_path: "cli/privkeys/ckb_key",
    bridge_typescript_path: "../ckb-contracts/build/release/eth-bridge-typescript",
    bridge_lockscript_path: "../ckb-contracts/build/release/eth-bridge-lockscript",
    light_client_typescript_path: "../ckb-contracts/build/release/eth-light-client-typescript",
    recipient_typescript_path: "../ckb-contracts/build/debug/eth-recipient-typescript",
    sudt_path: "cli/deps/simple_udt",
    eth_contract_address: "0x30F58CEEEC7Ace3fC2f938F13dD2C0c69B4A19b0",
    eth_token_address: "0xE9837e999934E2f61D4148CeEEb1Be3cc00Cb37c",
}
```



3、 初始化 ckb_light_client 合约
```shell script
RUST_LOG=info ./target/debug/force-eth-cli init-light-contract -i 4150 --private-key-path cli/privkeys/ckb_key --to $CKBChainDeployAddr  -f 500 -c 40000
```

```
InitLightContractArgs {
    init_height: 4150,
    to: "0xb698bB19b6e2346f513725A3Ce26AADdFCbA1C95",
    finalized_gc: 500,
    canonical_gc: 40000,
    eth_rpc_url: "http://172.18.0.51:8545",
    ckb_rpc_url: "http://127.0.0.1:8114",
    indexer_url: "http://127.0.0.1:8116",
    private_key_path: "cli/privkeys/ckb_key",
}
```

4、 开启 ckb-relay 

-  ckb-realy 是一个独立的服务，建议用单独的账户（之前准备了2个）进行 `relay ckb header`. 防止 nonce 冲突

```shell
RUST_LOG=info ./target/debug/force-eth-cli  ckb-relay  -k ~/account/relay-sk --per-amount 10 --to $CKBChainDeployAddr
```

```
CkbRelayArgs {
    gap: 20,  // 每次提交多少个块
    to: "0x3C049b35392D20e5C1E7bB6B32E26f07093A6923",
    private_key_path: "/home/fp/account/relay-sk",
    ckb_rpc_url: "http://localhost:8114",
    eth_rpc_url: "http://172.18.0.51:8545",
    indexer_rpc_url: "http://localhost:8116",
}
```



## 跨链步骤

### ETH ==> CKB

用户在 `ETH` 合约中 `lock` 数量为 N 的`token`，然后凭借 `lock hash` 可在 `CKB` `mint` 相对应的 `sudt token`



1、执行 approve： 用户使用其私钥 `eth_key` 对持有的 `erc20 token` 执行 `approve` 操作

```shell
RUST_LOG=debug ./target/debug/force-eth-cli approve --from $TokenLockerDepolyAddr --to $ERC20DeployAddr  -k cli/privkeys/eth_key
```

```
ApproveArgs {
    from: "0x30F58CEEEC7Ace3fC2f938F13dD2C0c69B4A19b0",
    to: "0xE9837e999934E2f61D4148CeEEb1Be3cc00Cb37c",
    rpc_url: "http://172.18.0.51:8545",
    private_key_path: "cli/privkeys/eth_key",
}
```



2、执行 lock-token： 用户将 `erc20` 锁入 `lock` 合约（即转账）

```shell
RUST_LOG=debug ./target/debug/force-eth-cli lock-token --to $TokenLockerDepolyAddr --token $ERC20DeployAddr --amount 100 --bridge-fee 10 --sudt-extra-data sudt_extra_data  -k cli/privkeys/eth_key
```

```
LockTokenArgs {
    to: "0x30F58CEEEC7Ace3fC2f938F13dD2C0c69B4A19b0",
    rpc_url: "http://172.18.0.51:8545",
    private_key_path: "/home/fp/account/sk",
    token: "0xE9837e999934E2f61D4148CeEEb1Be3cc00Cb37c",
    amount: 100,
    bridge_fee: 10,
    config_path: "/tmp/.force-bridge-cli/config.toml",
    sudt_extra_data: "sudt_extra_data",
    ckb_recipient_address: "ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37",
}
```

```
export LOCKHASH=0x81ec8d6d14fc5bbd573e70d2fb3aa162b7106e7f246d0a40209ecd059baec928
```



3、执行 mint ： 用户以私钥 `ckb_key` 提交 `lock tx hash` ，通过验证后会 mint 出与 `erc20` 对应的 `ckb sudt`

```shell
RUST_LOG=debug ./target/debug/force-eth-cli mint --hash $LOCKHASH --eth-contract-address $TokenLockerDepolyAddr --cell depend_on_eth_relay 
```

```
MintArgs {
    hash: "0x81ec8d6d14fc5bbd573e70d2fb3aa162b7106e7f246d0a40209ecd059baec928",
    eth_rpc_url: "http://172.18.0.51:8545",
    ckb_rpc_url: "http://127.0.0.1:8114",
    config_path: "/tmp/.force-bridge-cli/config.toml",
    indexer_url: "http://127.0.0.1:8116",
    private_key_path: "cli/privkeys/ckb_key",
    cell: "depend_on_eth_relay",
    eth_contract_address: "0x30F58CEEEC7Ace3fC2f938F13dD2C0c69B4A19b0",
}
```



4、查询余额进行验证： 余额无误即说明从 `ETH` 跨链到 `CKB` 成功

```shell
RUST_LOG=info  ./target/debug/force-eth-cli  query-sudt-blance --addr ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37 --token-addr $ERC20DeployAddr --lock-contract-addr $TokenLockerDepolyAddr
```

```
SudtGetBalanceArgs {
    config_path: "/tmp/.force-bridge-cli/config.toml",
    ckb_rpc_url: "http://localhost:8114",
    indexer_rpc_url: "http://localhost:8116",
    addr: "ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37",
    token_addr: "0x30F58CEEEC7Ace3fC2f938F13dD2C0c69B4A19b0",
    lock_contract_addr: "0xE9837e999934E2f61D4148CeEEb1Be3cc00Cb37c",
}
```



### CKB ==> ETH

用户 `burn`  数量为 X 的 `sudt token`，然后凭借 `burn tx proof ` 可在 `ETH` 合约 `unlock` 相对应的 `token`



5、执行 burn : 用户以私钥 `ckb_key`  提交一笔 `burn 15 sudt`  的交易，`relay proof` 的手续费是  `5 erc20 token`

```shell
RUST_LOG=info ./target/debug/force-eth-cli burn --burn-amount 15 --receive-addr 0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d --token-addr $ERC20DeployAddr   --lock-contract-addr $TokenLockerDepolyAddr --unlock-fee 5 --private-key-path cli/privkeys/ckb_key
```

```
BurnArgs {
    config_path: "/tmp/.force-bridge-cli/config.toml",
    tx_fee: "0.1",
    private_key_path: "cli/privkeys/ckb_key",
    ckb_rpc_url: "http://localhost:8114",
    eth_rpc_url: "http://localhost:8545",
    indexer_rpc_url: "http://localhost:8116",
    token_addr: "0xE9837e999934E2f61D4148CeEEb1Be3cc00Cb37c",
    receive_addr: "0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d",
    lock_contract_addr: "0x30F58CEEEC7Ace3fC2f938F13dD2C0c69B4A19b0",
    burn_amount: 15,
    unlock_fee: 5,
}
```

```shell
export BURNHASH=549b0c00a0f9adbea1dc9990698a68e92ed996e3fd203066298c3b3023339bd0
```



6、生成 tx proof ： 

```shell
./target/debug/force-eth-cli  generate-ckb-proof --tx-hash $BURNHASH
```

```shell
export BURNPROOF=aa0000001c0000001e0000002600000046000000660000008600000002003f150000000000003a9573671d4fad18b5f5e2618cc10e1d0c19f55dc32511926f6c7b4bdba116b8549b0c00a0f9adbea1dc9990698a68e92ed996e3fd203066298c3b3023339bd04b3e21bfc97b13ac99c13ce97148d568f86c0aece74000e87151e27d82b32dec01000000b2caa0a45584be5bb006166fbece708753f3e3e025d9c6498b27e500362b2c0b

export BURNINFO=5d0300001c00000020000000b8000000bc00000018010000e90200000000000004000000610b0d1269f5f2a761bd7d044778182719ceb58c1613a184083f39b26741b5d80000000000610b0d1269f5f2a761bd7d044778182719ceb58c1613a184083f39b26741b5d80300000000610b0d1269f5f2a761bd7d044778182719ceb58c1613a184083f39b26741b5d80400000000ace5ea83c478bb866edf122ff862085789158f5cbff155b7bb5f13058555b708000000000100000000020000000000000000000000930ab2fae6bdd9a293883eb12dfc3f09fbf6ee74097a8e8e70deac0c53dcf2cf010000000000000000000000cd0d11df99708f62bc3ec8854affcaa3d2739db5c92d2004bfbbc818153a09f302000000d101000010000000ba00000070010000aa000000100000001800000061000000007aa55404000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000c8328aabcd9b9e8e64fbc566c4385c3bdeb219d74900000010000000300000003100000017fb928d15bf98064304f2126f6b387e33d0c3c6572d293143ca373929ec3b5c001400000030f58ceeec7ace3fc2f938f13dd2c0c69b4a19b0b600000010000000180000006100000000c817a804000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000c8328aabcd9b9e8e64fbc566c4385c3bdeb219d755000000100000003000000031000000e1e354d6d643ad42724d40967e334984534e0367405c5ae42a9d7d63d77df41900200000007277baf7d9a986f612b929594e1c4b73e54f6613b98139e61817f4de1314843b610000001000000018000000610000005cce0a5c9a74b41b490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000c8328aabcd9b9e8e64fbc566c4385c3bdeb219d774000000100000005c0000007000000048000000403a53a7dfa7a4ab022e53feff11232b3140407de9837e999934e2f61d4148ceeeb1be3cc00cb37c0f00000000000000000000000000000005000000000000000000000000000000100000005500000000000000000000000000000000000000
```



7、执行 unlock： 用户以私钥 `eth_key` 提交包含 `burn tx proof`  的 `unlock` 交易

```shell
RUST_LOG=info ./target/debug/force-eth-cli  unlock --from f --to $TokenLockerDepolyAddr --tx-info $BURNPROOF --tx-proof $BURNPROOF -k cli/privkeys/eth_key
```

```
UnlockArgs {
    from: "f",
    to: "0x30F58CEEEC7Ace3fC2f938F13dD2C0c69B4A19b0",
    private_key_path: "cli/privkeys/eth_key",
    tx_proof: "aa0000001c0000001e0000002600000046000000660000008600000002003f150000000000003a9573671d4fad18b5f5e2618cc10e1d0c19f55dc32511926f6c7b4bdba116b8549b0c00a0f9adbea1dc9990698a68e92ed996e3fd203066298c3b3023339bd04b3e21bfc97b13ac99c13ce97148d568f86c0aece74000e87151e27d82b32dec01000000b2caa0a45584be5bb006166fbece708753f3e3e025d9c6498b27e500362b2c0b",
    tx_info: "5d0300001c00000020000000b8000000bc00000018010000e90200000000000004000000610b0d1269f5f2a761bd7d044778182719ceb58c1613a184083f39b26741b5d80000000000610b0d1269f5f2a761bd7d044778182719ceb58c1613a184083f39b26741b5d80300000000610b0d1269f5f2a761bd7d044778182719ceb58c1613a184083f39b26741b5d80400000000ace5ea83c478bb866edf122ff862085789158f5cbff155b7bb5f13058555b708000000000100000000020000000000000000000000930ab2fae6bdd9a293883eb12dfc3f09fbf6ee74097a8e8e70deac0c53dcf2cf010000000000000000000000cd0d11df99708f62bc3ec8854affcaa3d2739db5c92d2004bfbbc818153a09f302000000d101000010000000ba00000070010000aa000000100000001800000061000000007aa55404000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000c8328aabcd9b9e8e64fbc566c4385c3bdeb219d74900000010000000300000003100000017fb928d15bf98064304f2126f6b387e33d0c3c6572d293143ca373929ec3b5c001400000030f58ceeec7ace3fc2f938f13dd2c0c69b4a19b0b600000010000000180000006100000000c817a804000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000c8328aabcd9b9e8e64fbc566c4385c3bdeb219d755000000100000003000000031000000e1e354d6d643ad42724d40967e334984534e0367405c5ae42a9d7d63d77df41900200000007277baf7d9a986f612b929594e1c4b73e54f6613b98139e61817f4de1314843b610000001000000018000000610000005cce0a5c9a74b41b490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000c8328aabcd9b9e8e64fbc566c4385c3bdeb219d774000000100000005c0000007000000048000000403a53a7dfa7a4ab022e53feff11232b3140407de9837e999934e2f61d4148ceeeb1be3cc00cb37c0f00000000000000000000000000000005000000000000000000000000000000100000005500000000000000000000000000000000000000",
    eth_rpc_url: "http://172.18.0.51:8545",
}
```



8、 第 5 ~ 7 条可合并成一条命令执行

```shell
RUST_LOG=info ./target/debug/force-eth-cli transfer-from-ckb --burn-amount 5 --unlock-fee 1 --ckb-privkey-path cli/privkeys/ckb_key --eth-privkey-path cli/privkeys/eth_key --receive-addr 0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d   --token-addr $ERC20DeployAddr --lock-contract-addr $TokenLockerDepolyAddr --light-client-addr $CKBChainDeployAddr
```

```
TransferFromCkbArgs {
    config_path: "/tmp/.force-bridge-cli/config.toml",
    tx_fee: "0.1",
    ckb_privkey_path: "cli/privkeys/ckb_key",
    eth_privkey_path: "cli/privkeys/eth_key",
    ckb_rpc_url: "http://localhost:8114",
    eth_rpc_url: "http://172.18.0.51:8545",
    indexer_rpc_url: "http://localhost:8116",
    token_addr: "0xE9837e999934E2f61D4148CeEEb1Be3cc00Cb37c",
    receive_addr: "0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d",
    lock_contract_addr: "0x30F58CEEEC7Ace3fC2f938F13dD2C0c69B4A19b0",
    light_client_addr: "0x3C049b35392D20e5C1E7bB6B32E26f07093A6923",
    burn_amount: 5,
    unlock_fee: 1,
},
```

