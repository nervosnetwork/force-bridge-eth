# force bridge dapp server api

- [force bridge dapp server 文档](#force-bridge-dapp-server-文档)
  - [API 接口文档](#api-接口文档)
    - [settings](#settings)
    - [init_token](#init_token)
    - [lock](#lock)
    - [burn](#burn)
    - [get_eth_to_ckb_status](#get_eth_to_ckb_status) 
    - [get_ckb_to_eth_status](#get_ckb_to_eth_status)
    - [get_crosschain_history](#get_crosschain_history)
    - [get_sudt_balance](#get_sudt_balance)

## API 接口文档

### settings

- path: `/settings`
- method: get

> 后文中的标题即为 path，方法默认为 post

获取 force bridge settings。前端可以保存该配置备用。

```json
{
    "eth_token_locker_addr": "0xcD62E77cFE0386343c15C13528675aae9925D7Ae",
    "eth_ckb_chain_addr": "0x8326e1d621Cd32752920ed2A44B49bB1a96c7391",
    "bridge_lockscript": {
        "code_hash": "da6d3ad483fb11a5619e65035c3139acdb17c26e73647b7f0ac62a4036ca4e72",
        "hash_type": 0,
        "outpoint": {
            "tx_hash": "49188fdc4c2185bed57f2aeaf2fac480444d58e395de7eb6db33e69acacdcd7d",
            "index": 0,
            "dep_type": 0
        }
    },
    "bridge_typescript": {
        "code_hash": "5b6cc072f1a36ddc5dc2bfe76cb806fb886e9b22a5d4f389e865dd34d8c0d957",
        "hash_type": 0,
        "outpoint": {
            "tx_hash": "49188fdc4c2185bed57f2aeaf2fac480444d58e395de7eb6db33e69acacdcd7d",
            "index": 1,
            "dep_type": 0
        }
    },
    "light_client_typescript": {
        "code_hash": "2e8ca901a6283b8d51a2c083c6756bea2e15a5b78a408b8f3544fad402cc6980",
        "hash_type": 0,
        "outpoint": {
            "tx_hash": "49188fdc4c2185bed57f2aeaf2fac480444d58e395de7eb6db33e69acacdcd7d",
            "index": 3,
            "dep_type": 0
        }
    },
    "recipient_typescript": {
        "code_hash": "ed0df97ea89ce848b20479194c9eb50cda612837f2db516b828ffeea61473ff3",
        "hash_type": 0,
        "outpoint": {
            "tx_hash": "49188fdc4c2185bed57f2aeaf2fac480444d58e395de7eb6db33e69acacdcd7d",
            "index": 2,
            "dep_type": 0
        }
    },
    "simple_bridge_typescript": {
        "code_hash": "b093c89a824a4ec3492b59db4fdd25296e2e15e0e89813cf4c29c2ff1173ba8c",
        "hash_type": 0,
        "outpoint": {
            "tx_hash": "49188fdc4c2185bed57f2aeaf2fac480444d58e395de7eb6db33e69acacdcd7d",
            "index": 4,
            "dep_type": 0
        }
    },
    "sudt": {
        "code_hash": "e1e354d6d643ad42724d40967e334984534e0367405c5ae42a9d7d63d77df419",
        "hash_type": 0,
        "outpoint": {
            "tx_hash": "49188fdc4c2185bed57f2aeaf2fac480444d58e395de7eb6db33e69acacdcd7d",
            "index": 5,
            "dep_type": 0
        }
    },
    "light_client_cell_script": {
        "cell_script": "590000001000000030000000310000002e8ca901a6283b8d51a2c083c6756bea2e15a5b78a408b8f3544fad402cc698000240000005edca2d744b6eaa347de7ff0edcd2e6e88ab8f2836bcbd0df0940026956e5f8107000000"
    },
    "multisig_address": {
        "addresses": [
            "ckt1qyqyph8v9mclls35p6snlaxajeca97tc062sa5gahk",
            "ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37"
        ],
        "require_first_n": 0,
        "threshold": 2
    },
    "ckb_relay_mutlisig_threshold": {
        "threshold": 1
    },
    "pw_locks": {
        "inner": []
    }
}
```

### init_token

- path: `/init_token`
- method: post

Dapp 在支持某种 token 跨链前，需调用该接口传入 token_address，server 收到请求后在后端创建该种 token 的 replay resist cells。Init token 成功后，用户可在 lock token 时自动使用 replay resist cell。

> 目前该接口只支持串行访问。比如 Dapp 支持 4 种 token 跨链，则 Dapp 需要串行调用该接口 4 次。
> 后续我们会改进该接口，以支持并发访问。

输入参数示例：
```json
{
  "token_address": "0000000000000000000000000000000000000000"
}
```

#### 输入参数说明
- eth_token_address：跨链 erc20 token 地址(去除 0x)。eth 为 0000000000000000000000000000000000000000。


#### 结果说明
- 成功返回 200 code
- 重复 init 失败返回 400 code
```
// 重复 init 失败 的 response body
bad request data: token already inited
```


### lock

获取用户锁定资产到以太坊跨链合约的交易。

输入参数：
```json
{
  "token_address": "0000000000000000000000000000000000000000",
  "amount": "0x100",
  "bridge_fee": "0x0",
  "sender": "0x42e8763917A72e07369AD54B158b0FA839f060bc",
  "ckb_recipient_address": "ckt1qyqdtmf8yy3agyarxpwre60ydrl8082n5hysd4nn86",
  "sudt_extra_data": "bbc62a3943831e918f864a0306bc794b9e2724afc141b69d47e070f1f5686d4200000000",
  "gas_price": "0x1",
  "nonce": "0x1"
}
```

#### 输入参数说明
- token_address：要跨链的 token 地址（去除 0x）。eth 为 0000000000000000000000000000000000000000。
- amount：要跨链的数量，最大为 u128，hex 格式。
- bridge_fee：跨链手续费，从要跨的币中扣除。最大不超过 amount, hex 格式。
- sender: 发送该交易的地址，用来估计 gas 费。为用户钱包的以太坊地址。
- ckb_recipient_address：ckb 接收跨链资产地址。
- sudt_extra_data：生成的 sudt 中的额外数据，hex bytes。
- gas_price：发送 lock 交易的 gas 费用， hex 格式。
- nonce：交易发起地址的 nonce，hex 格式。

#### 结果说明

```json
{
    "nonce": "0x1",
    "to": "0xcd62e77cfe0386343c15c13528675aae9925d7ae",
    "value": "0x100",
    "gas_price": "0x1",
    "gas": "0xbd49",
    "data": "10b214ec00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000049490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000d5ed272123d413a3305c3ce9e468fe779d53a5c90000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002485ffef8b066255fb2942f78afeda09643b0b1cfde1be0acfc1dc3421236b85c404000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000024bbc62a3943831e918f864a0306bc794b9e2724afc141b69d47e070f1f5686d420000000000000000000000000000000000000000000000000000000000000000",
    "raw": "010182bd4994cd62e77cfe0386343c15c13528675aae9925d7ae820100b901c410b214ec00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000049490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000d5ed272123d413a3305c3ce9e468fe779d53a5c90000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002485ffef8b066255fb2942f78afeda09643b0b1cfde1be0acfc1dc3421236b85c404000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000024bbc62a3943831e918f864a0306bc794b9e2724afc141b69d47e070f1f5686d420000000000000000000000000000000000000000000000000000000000000000"
}
```

- raw 为以太坊未签名原始交易
- 其它字段为人类可读的以太坊交易字段

### burn

用户在 ckb 上 burn 掉 mirror token，跨链回 eth。前端使用该接口获取未签名的 burn 交易。

```json
{
  "from_lockscript_addr": "ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4",
  "tx_fee": "0.01",
  "unlock_fee": "0x0",
  "amount": "0x10",
  "token_address": "0x0000000000000000000000000000000000000000",
  "recipient_address": "0x728060aB319E06AC6b5d5c0C74009F7D1f5730DB"
}
```

#### 输入参数说明
- from_lockscript_addr：要执行 burn 交易的 ckb 地址
- tx_fee：交易手续费。单位为 ckb，支持小数。
- amount：跨链金额，hex 格式，最大不超过 u128。
- unlock_fee：跨链手续费，hex 格式，最大不超过 amount 的值。
- token_address：ckb mirror token 的原 erc20 地址
- recipient_address：接收跨链金额的以太坊地址

```json
{
    "raw_tx": {
        "version": "0x0",
        "cell_deps": [
            {
                "out_point": {
                    "tx_hash": "0x49188fdc4c2185bed57f2aeaf2fac480444d58e395de7eb6db33e69acacdcd7d",
                    "index": "0x0"
                },
                "dep_type": "code"
            },
            {
                "out_point": {
                    "tx_hash": "0x49188fdc4c2185bed57f2aeaf2fac480444d58e395de7eb6db33e69acacdcd7d",
                    "index": "0x2"
                },
                "dep_type": "code"
            },
            {
                "out_point": {
                    "tx_hash": "0x49188fdc4c2185bed57f2aeaf2fac480444d58e395de7eb6db33e69acacdcd7d",
                    "index": "0x5"
                },
                "dep_type": "code"
            },
            {
                "out_point": {
                    "tx_hash": "0xa777fd1964ffa98a7b0b6c09ff71691705d84d5ed1badfb14271a3a870bdd06b",
                    "index": "0x0"
                },
                "dep_type": "dep_group"
            }
        ],
        "header_deps": [],
        "inputs": [
            {
                "since": "0x0",
                "previous_output": {
                    "tx_hash": "0x31900669ba31ab2f14931eb35d9e74a024dcdff8c7a06c521cd6f7c1516cf838",
                    "index": "0x1"
                }
            },
            {
                "since": "0x0",
                "previous_output": {
                    "tx_hash": "0x0727e4a50e70f44ab3392787430d5150939ee91577a245558407bc2ad6433f2d",
                    "index": "0x1"
                }
            }
        ],
        "outputs": [
            {
                "capacity": "0x5d21dba00",
                "lock": {
                    "code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
                    "hash_type": "type",
                    "args": "0xa4bf8e4c7f6f65f35dd3cc30c8fc45c8e99a171c"
                },
                "type": {
                    "code_hash": "0xed0df97ea89ce848b20479194c9eb50cda612837f2db516b828ffeea61473ff3",
                    "hash_type": "data",
                    "args": "0x"
                }
            },
            {
                "capacity": "0x4a817c800",
                "lock": {
                    "code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
                    "hash_type": "type",
                    "args": "0xa4bf8e4c7f6f65f35dd3cc30c8fc45c8e99a171c"
                },
                "type": {
                    "code_hash": "0xe1e354d6d643ad42724d40967e334984534e0367405c5ae42a9d7d63d77df419",
                    "hash_type": "data",
                    "args": "0xb5ff94e85f04396cf5b852446eb75d8880cad4d94a1c17d0e5cd70470e6c2ba8"
                }
            },
            {
                "capacity": "0xe3027813c0",
                "lock": {
                    "code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
                    "hash_type": "type",
                    "args": "0xa4bf8e4c7f6f65f35dd3cc30c8fc45c8e99a171c"
                },
                "type": null
            }
        ],
        "outputs_data": [
            "0x728060ab319e06ac6b5d5c0c74009f7d1f5730db0000000000000000000000000000000000000000cd62e77cfe0386343c15c13528675aae9925d7ae88d9ffc645fef37c2097140cdc2923726d4efe16131e76e85757b446138e39ceda6d3ad483fb11a5619e65035c3139acdb17c26e73647b7f0ac62a4036ca4e721000000000000000000000000000000000000000000000000000000000000000",
            "0x52000000000000000000000000000000",
            "0x"
        ],
        "witnesses": [],
        "hash": "0x9202af33f0a242c54603e7d4e977b74b9ac6b25a3c69dd4661123b8882670a1f"
    }
}
```

#### 结果说明
- raw_tx: ckb 原始交易。格式与 ckb rpc 接口一致。用户签名后可以自行发交易。

### get_eth_to_ckb_status

通过 eth_lock_tx_hash 获取跨链状态

#### 输入参数说明

```json
{
    "eth_lock_tx_hash": "afc74282409140b853b3cbb74d772bc835e7ea5643704d35db77b8c306ed5fe0"
}
```

- eth_lock_tx_hash: 以太坊 lock 交易的哈希（去除 0x)

#### 结果说明

```json
{
    "eth_lock_tx_hash": "afc74282409140b853b3cbb74d772bc835e7ea5643704d35db77b8c306ed5fe0",
    "status": "pending",
    "err_msg": "",
    "token_addr": "0000000000000000000000000000000000000000",
    "sender_addr": "17c4b5ce0605f63732bfd175fece7ac6b4620fd2",
    "locked_amount": "0x64",
    "bridge_fee": "0x0",
    "ckb_recipient_lockscript": "490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000a4bf8e4c7f6f65f35dd3cc30c8fc45c8e99a171c",
    "sudt_extra_data": "737564745f65787472615f64617461",
    "ckb_tx_hash": null,
    "block_number": 607,
    "replay_resist_outpoint": "85ffef8b066255fb2942f78afeda09643b0b1cfde1be0acfc1dc3421236b85c407000000"
}
```

- status:
    - success
    - pending
    - irreparable error
- err_msg: 报错信息

### get_ckb_to_eth_status

通过 ckb_burn_tx_hash 获取跨链状态

#### 输入参数说明

```json
{
    "ckb_burn_tx_hash": "31900669ba31ab2f14931eb35d9e74a024dcdff8c7a06c521cd6f7c1516cf838"
}
```

- ckb_burn_tx_hash: ckb burn 交易的哈希（去除 0x）

#### 结果说明

```json
{
    "id": 1,
    "ckb_burn_tx_hash": "31900669ba31ab2f14931eb35d9e74a024dcdff8c7a06c521cd6f7c1516cf838",
    "status": "success",
    "recipient_addr": "403a53a7dfa7a4ab022e53feff11232b3140407d",
    "token_addr": "0000000000000000000000000000000000000000",
    "token_amount": "0x2",
    "fee": "0x1",
    "eth_tx_hash": "6302e20bf9b1ed24a9044d82b97c583477cd9a8aa8ae4e2ee27d0bf43e97613a",
    "ckb_block_number": 720,
    "eth_block_number": 830
}
```

- status:
    - success
    - pending

### get_crosschain_history

#### 输入参数说明

```json
{
    "eth_recipient_addr": "403A53A7Dfa7a4AB022e53FeFf11232b3140407d",
    "ckb_recipient_lockscript_addr": "ckt1qyq2f0uwf3lk7e0nthfucvxgl3zu36v6zuwq6mlzps"
}
```

- eth_recipient_addr: 以太坊接收地址(去除 0x)
- ckb_recipient_lockscript_addr: ckb 接收者的 lockscript address 地址

#### 结果说明

```json
{
    "eth_to_ckb": [
        {
            "id": 1,
            "eth_tx_hash": "0xafc74282409140b853b3cbb74d772bc835e7ea5643704d35db77b8c306ed5fe0",
            "ckb_tx_hash": null,
            "status": "success",
            "sort": "eth_to_ckb",
            "amount": "0x64",
            "token_addr": "0x0000000000000000000000000000000000000000"
        }
    ],
    "ckb_to_eth": [
        {
            "id": 1,
            "eth_tx_hash": "0x6302e20bf9b1ed24a9044d82b97c583477cd9a8aa8ae4e2ee27d0bf43e97613a",
            "ckb_tx_hash": "0x31900669ba31ab2f14931eb35d9e74a024dcdff8c7a06c521cd6f7c1516cf838",
            "status": "success",
            "sort": "ckb_to_eth",
            "amount": "0x2",
            "token_addr": "0x0000000000000000000000000000000000000000"
        }
    ]
}
```

- eth_tx_hash: 跨链挂单的以太坊锁定资产交易 hash
- ckb_tx_hash: 跨链挂单的 ckb 交易 hash
- status:
    - success
    - pending
- sort: 类型为 ckb_to_eth 或者 eth_to_ckb
- amount: 跨链金额，hex 格式
- token_addr: 跨链涉及的币种

### get_sudt_balance

获取 sudt 余额

```json
{
  "address": "ckt1qyq2f0uwf3lk7e0nthfucvxgl3zu36v6zuwq6mlzps",
  "token_address": "0x0000000000000000000000000000000000000000"
}
```

#### 输入参数说明
- token_address：erc20 地址
- address：ckb 地址

```json
{
    "balance": "0x62",
    "sudt_script": {
        "args": "0xb5ff94e85f04396cf5b852446eb75d8880cad4d94a1c17d0e5cd70470e6c2ba8",
        "code_hash": "0xe1e354d6d643ad42724d40967e334984534e0367405c5ae42a9d7d63d77df419",
        "hash_type": "data"
    }
}
```

#### 结果说明
- balance：用户 token_address erc20 在 ckb 上的 mirror token 余额
