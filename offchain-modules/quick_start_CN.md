## 环境准备

### 运行 ETH 和 CKB节点

[docker-compose 运行](https://github.com/nervosnetwork/force-bridge-eth/tree/main/docker) 即可 ，会运行 `ckb`,   `ckb-indexer`,  `eth`  三个服务

### 项目构建
#### ETH contract
```shell script
$ cd eth-contracts
$ npm install
```
#### CKB contract
```shell script
$ cd ckb-contracts
$ capsule build --release
```

#### off-chain client
```shell script
$ cd offchain-modules
$ cargo build
```

### 账户说明

| 私钥                                                         | 地址                                             | 用途说明                               | 修改方式                                                        |
| ------------------------------------------------------------ | ------------------------------------------------ | ---------------------------------- | ------------------------------------------------------------ |
| `0xc4ad657963930fbff2e9de3404b30a4e21432c89952ed430b56bf802945ed37a` | `0x17c4b5CE0605F63732bfd175feCe7aC6b4620FD2`     | 执行 `lock unlock` 等`eth`合约操作  | 在 eth 的配置文件  [genesis.json](https://github.com/nervosnetwork/force-bridge-eth/blob/dedf9f05ccbfb606633e074d5f8789b5e9eb74be/docker/geth/geth-genesis.json#L14) 中修改 |
| `0x719e94ec5d2ecef67b5878503ffd6e1e0e2fe7a52ddd55c436878cb4d52d376d` | `0x8951a3DdEf2bB36fF3846C3B6968812C269f4561`     | `relay_ckb_header`                 | 在 eth 的配置文件  [genesis.json](https://github.com/nervosnetwork/force-bridge-eth/blob/dedf9f05ccbfb606633e074d5f8789b5e9eb74be/docker/geth/geth-genesis.json#L14) 中修改 |
| `0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc` | `ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37` | 执行 `mint burn`  等`ckb` 合约操作 | 默认生成并保存在 spec/dev.toml                               |

#### 保存私钥

```shell
mkdir /tmp/privkeys

echo "c4ad657963930fbff2e9de3404b30a4e21432c89952ed430b56bf802945ed37a" > /tmp/privkeys/eth_key

echo "719e94ec5d2ecef67b5878503ffd6e1e0e2fe7a52ddd55c436878cb4d52d376d" > /tmp/privkeys/relay_key

echo "d00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc" > /tmp/privkeys/ckb_key
```



### 合约部署

1、部署 ETH 合约

执行命令和结果：`hardhat.config.js` 中的私钥地址 就是后续 跨链操作的地址（部署合约才有 erc20）

```shell
➜ eth-contracts $ npx hardhat run scripts/geth/deployAll.js --network geth
```

部署后，设置对应环境变量，方便后续操作，具体对应如下

```shell
export ERC20DeployAddr=0x6Aa95f56c83C44619aBeed3EAE7B4145C897ef43
export CKBChainDeployAddr=0x73dBDC911Fc128790cf29818c94FBcFd27Ba3DEF
export TokenLockerDepolyAddr=0x7D67911B516e79ED2011e60D71581c89fE133AA9
```



2、部署  CKB 合约 

```shell
➜ offchain-modules $ mkdir /tmp/.force-bridge-cli
➜ offchain-modules $ RUST_LOG=info ./target/debug/force-eth-cli dev-init --eth-contract-address $TokenLockerDepolyAddr --eth-token-address $ERC20DeployAddr --private-key-path /tmp/privkeys/ckb_key -f
```

```
DevInitArgs {
    force: true,
    config_path: "/tmp/.force-bridge-cli/config.toml",
    rpc_url: "http://127.0.0.1:8114",
    indexer_url: "http://127.0.0.1:8116",
    private_key_path: "/tmp/privkeys/ckb_key",
    bridge_typescript_path: "../ckb-contracts/build/release/eth-bridge-typescript",
    bridge_lockscript_path: "../ckb-contracts/build/release/eth-bridge-lockscript",
    light_client_typescript_path: "../ckb-contracts/build/release/eth-light-client-typescript",
    recipient_typescript_path: "../ckb-contracts/build/release/eth-recipient-typescript",
    sudt_path: "cli/deps/simple_udt",
    eth_contract_address: "0x7D67911B516e79ED2011e60D71581c89fE133AA9",
    eth_token_address: "0x6Aa95f56c83C44619aBeed3EAE7B4145C897ef43",
}
```



3、 初始化 ckb_light_client 合约
```shell script
➜ offchain-modules $ RUST_LOG=info ./target/debug/force-eth-cli init-ckb-light-contract -i 16424 --private-key-path /tmp/privkeys/eth_key --to $CKBChainDeployAddr  -f 500 -c 40000
```

```
InitLightContractArgs {
    init_height: 16424,
    to: "0x73dBDC911Fc128790cf29818c94FBcFd27Ba3DEF",
    finalized_gc: 500,
    canonical_gc: 40000,
    eth_rpc_url: "http://172.18.0.51:8545",
    ckb_rpc_url: "http://127.0.0.1:8114",
    indexer_url: "http://127.0.0.1:8116",
    private_key_path: "/tmp/privkeys/eth_key",
    gas_price: 0,
}
```

4、 开启 ckb-relay 

-  ckb-realy 是一个独立的服务，建议用单独的账户（之前准备了2个）进行 `relay ckb header`. 防止 nonce 冲突

```shell
➜ offchain-modules $ RUST_LOG=info ./target/debug/force-eth-cli  ckb-relay  -k /tmp/privkeys/relay_key --per-amount 20 --to $CKBChainDeployAddr 
```

```
CkbRelayArgs {
    per_amount: 20,
    to: "0x73dBDC911Fc128790cf29818c94FBcFd27Ba3DEF",
    private_key_path: "/tmp/privkeys/relay_key",
    ckb_rpc_url: "http://localhost:8114",
    eth_rpc_url: "http://172.18.0.51:8545",
    indexer_rpc_url: "http://localhost:8116",
    gas_price: 0
}
```



## 跨链步骤

### ETH ==> CKB

用户在 `ETH` 合约中 `lock` 数量为 N 的`token`，然后凭借 `lock hash` 可在 `CKB` `mint` 相对应的 `sudt token`



1、执行 approve： 用户使用其私钥 `eth_key` 对持有的 `erc20 token` 执行 `approve` 操作

```shell
➜ offchain-modules $ RUST_LOG=info ./target/debug/force-eth-cli approve --from $TokenLockerDepolyAddr --to $ERC20DeployAddr  -k /tmp/privkeys/eth_key 
```

```
ApproveArgs {
    from: "0x7D67911B516e79ED2011e60D71581c89fE133AA9",
    to: "0x6Aa95f56c83C44619aBeed3EAE7B4145C897ef43",
    rpc_url: "http://172.18.0.51:8545",
    private_key_path: "/tmp/privkeys/eth_key",
    gas_price: 0
}
```



2、执行 lock-token： 用户将 `erc20` 锁入 `lock` 合约（即转账）

```shell
➜ offchain-modules $ RUST_LOG=info  ./target/debug/force-eth-cli lock-token --to $TokenLockerDepolyAddr --token $ERC20DeployAddr --amount 1000 --bridge-fee 1 --sudt-extra-data sudt_extra_data  -k /tmp/privkeys/eth_key
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
➜ offchain-modules $ RUST_LOG=debug ./target/debug/force-eth-cli mint --hash $LOCKHASH --eth-contract-address $TokenLockerDepolyAddr --cell depend_on_eth_relay  --private-key-path /tmp/privkeys/ckb_key
```

```
MintArgs {
    hash: "0xd549f0b1d37009f8080a1df1636f3a1e6ceebad4f4ba7f3b70c846cef6f0b116",
    eth_rpc_url: "http://172.18.0.51:8545",
    ckb_rpc_url: "http://127.0.0.1:8114",
    config_path: "/tmp/.force-bridge-cli/config.toml",
    indexer_url: "http://127.0.0.1:8116",
    private_key_path: "/tmp/privkeys/ckb_key",
    cell: "depend_on_eth_relay",
    eth_contract_address: "0x7D67911B516e79ED2011e60D71581c89fE133AA9",
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
➜ offchain-modules $ RUST_LOG=info ./target/debug/force-eth-cli burn --burn-amount 15 --receive-addr 0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d --token-addr $ERC20DeployAddr   --lock-contract-addr $TokenLockerDepolyAddr --unlock-fee 5 --private-key-path /tmp/privkeys/ckb_key
```

```
BurnArgs {
    config_path: "/tmp/.force-bridge-cli/config.toml",
    tx_fee: "0.1",
    private_key_path: "/tmp/privkeys/ckb_key",
    ckb_rpc_url: "http://localhost:8114",
    eth_rpc_url: "http://172.18.0.51:8545",
    indexer_rpc_url: "http://localhost:8116",
    token_addr: "0x6Aa95f56c83C44619aBeed3EAE7B4145C897ef43",
    receive_addr: "0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d",
    lock_contract_addr: "0x7D67911B516e79ED2011e60D71581c89fE133AA9",
    burn_amount: 15,
    unlock_fee: 5,
}
```

```shell
export BURNHASH=549b0c00a0f9adbea1dc9990698a68e92ed996e3fd203066298c3b3023339bd0
```



6、生成 tx proof ： 

```shell
➜ offchain-modules $ ./target/debug/force-eth-cli  generate-ckb-proof --tx-hash $BURNHASH
```

```shell
export BURNPROOF=aa0000001c0000001e0000002600000046000000660000008600000002003f150000000000003a9573671d4fad18b5f5e2618cc10e1d0c19f55dc32511926f6c7b4bdba116b8549b0c00a0f9adbea1dc9990698a68e92ed996e3fd203066298c3b3023339bd04b3e21bfc97b13ac99c13ce97148d568f86c0aece74000e87151e27d82b32dec01000000b2caa0a45584be5bb006166fbece708753f3e3e025d9c6498b27e500362b2c0b

export BURNINFO=5d0300001c00000020000000b8000000bc00000018010000e90200000000000004000000610b0d1269f5f2a761bd7d044778182719ceb58c1613a184083f39b26741b5d80000000000610b0d1269f5f2a761bd7d044778182719ceb58c1613a184083f39b26741b5d80300000000610b0d1269f5f2a761bd7d044778182719ceb58c1613a184083f39b26741b5d80400000000ace5ea83c478bb866edf122ff862085789158f5cbff155b7bb5f13058555b708000000000100000000020000000000000000000000930ab2fae6bdd9a293883eb12dfc3f09fbf6ee74097a8e8e70deac0c53dcf2cf010000000000000000000000cd0d11df99708f62bc3ec8854affcaa3d2739db5c92d2004bfbbc818153a09f302000000d101000010000000ba00000070010000aa000000100000001800000061000000007aa55404000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000c8328aabcd9b9e8e64fbc566c4385c3bdeb219d74900000010000000300000003100000017fb928d15bf98064304f2126f6b387e33d0c3c6572d293143ca373929ec3b5c001400000030f58ceeec7ace3fc2f938f13dd2c0c69b4a19b0b600000010000000180000006100000000c817a804000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000c8328aabcd9b9e8e64fbc566c4385c3bdeb219d755000000100000003000000031000000e1e354d6d643ad42724d40967e334984534e0367405c5ae42a9d7d63d77df41900200000007277baf7d9a986f612b929594e1c4b73e54f6613b98139e61817f4de1314843b610000001000000018000000610000005cce0a5c9a74b41b490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000c8328aabcd9b9e8e64fbc566c4385c3bdeb219d774000000100000005c0000007000000048000000403a53a7dfa7a4ab022e53feff11232b3140407de9837e999934e2f61d4148ceeeb1be3cc00cb37c0f00000000000000000000000000000005000000000000000000000000000000100000005500000000000000000000000000000000000000
```



7、执行 unlock： 用户以私钥 `eth_key` 提交包含 `burn tx proof`  的 `unlock` 交易

```shell
➜ offchain-modules $ RUST_LOG=info ./target/debug/force-eth-cli  unlock --to $TokenLockerDepolyAddr --tx-info $BURNINFO --tx-proof $BURNPROOF -k /tmp/privkeys/eth_key 
```

```
UnlockArgs {
    to: "0x7D67911B516e79ED2011e60D71581c89fE133AA9",
    private_key_path: "/tmp/privkeys/eth_key",
    tx_proof: "aa0000001c0000001e00000026000000460000006600000086000000020023410000000000008867f1dfec21dfe275d6cbefb25f31ac3d73dda07652acd91b8f2f82f7b1f4ae9a6c12fef9245a04fc1a7e5174d9e0b9d6936c5bb121cb155881923e5bd38678bd3294137f2c199ae728a656b7ff549e675da929dff37707df54cb633314ebf5010000008bb49b9e7fe7ddf649df2b067155669591e4fa64bcd1ade0467b6d1af532aece",
    tx_info: "5d0300001c00000020000000b8000000bc00000018010000e90200000000000004000000f072a96b421f144d88636621bea26cdb08981200a64c7a4e415a59d145eef5d50000000000f072a96b421f144d88636621bea26cdb08981200a64c7a4e415a59d145eef5d50300000000f072a96b421f144d88636621bea26cdb08981200a64c7a4e415a59d145eef5d50400000000ace5ea83c478bb866edf122ff862085789158f5cbff155b7bb5f13058555b70800000000010000000002000000000000000000000063ca126bf08587e21bb419f989d9e53da917e1e5f74b47c0bb1cdcd94d0904a901000000000000000000000063ca126bf08587e21bb419f989d9e53da917e1e5f74b47c0bb1cdcd94d0904a902000000d101000010000000ba00000070010000aa000000100000001800000061000000007aa55404000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000c8328aabcd9b9e8e64fbc566c4385c3bdeb219d74900000010000000300000003100000094ad18d0329e195bb0a49c2f46ceb93ab50884331ad55f6df6ed012c1cc2d3bd00140000007d67911b516e79ed2011e60d71581c89fe133aa9b600000010000000180000006100000000c817a804000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000c8328aabcd9b9e8e64fbc566c4385c3bdeb219d755000000100000003000000031000000e1e354d6d643ad42724d40967e334984534e0367405c5ae42a9d7d63d77df4190020000000d0a7bba01cde7a30d17e41ffad538c0b62e58c3d82a210771b00e0027b7aa16d61000000100000001800000061000000ce2adbc72860b41b490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000c8328aabcd9b9e8e64fbc566c4385c3bdeb219d774000000100000005c0000007000000048000000403a53a7dfa7a4ab022e53feff11232b3140407d6aa95f56c83c44619abeed3eae7b4145c897ef430f00000000000000000000000000000005000000000000000000000000000000100000005500000000000000000000000000000000000000",
    eth_rpc_url: "http://172.18.0.51:8545",
    gas_price: 0,
}
```



8、 第 5 ~ 7 条可合并成一条命令执行

```shell
➜ offchain-modules $ RUST_LOG=info ./target/debug/force-eth-cli transfer-from-ckb --burn-amount 10 --unlock-fee 1 --ckb-privkey-path /tmp/privkeys/ckb_key --eth-privkey-path /tmp/privkeys/eth_key  --receive-addr 0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d   --token-addr $ERC20DeployAddr --lock-contract-addr $TokenLockerDepolyAddr --light-client-addr $CKBChainDeployAddr
```

```
TransferFromCkbArgs {
    config_path: "/tmp/.force-bridge-cli/config.toml",
    tx_fee: "0.1",
    ckb_privkey_path: "/tmp/privkeys/ckb_key",
    eth_privkey_path: "/tmp/privkeys/eth_key",
    ckb_rpc_url: "http://localhost:8114",
    eth_rpc_url: "http://172.18.0.51:8545",
    indexer_rpc_url: "http://localhost:8116",
    token_addr: "0x6Aa95f56c83C44619aBeed3EAE7B4145C897ef43",
    receive_addr: "0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d",
    lock_contract_addr: "0x7D67911B516e79ED2011e60D71581c89fE133AA9",
    light_client_addr: "0x73dBDC911Fc128790cf29818c94FBcFd27Ba3DEF",
    burn_amount: 10,
    unlock_fee: 1,
    gas_price: 0,
},
```

