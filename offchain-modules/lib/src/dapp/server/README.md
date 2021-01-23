# force bridge dapp server api

- [force bridge dapp server 文档](#force-bridge-dapp-server-文档)
  - [API 接口文档](#api-接口文档)
    - [settings](#settings)
    - [init_token](#init_token)
    - [lock](#lock)
      - [输入参数说明](#输入参数说明-1)
      - [结果说明](#结果说明-1)
      - [前端自行组装交易指南](#前端自行组装交易指南)
    - [burn](#burn)
      - [输入参数说明](#输入参数说明-3)
      - [结果说明](#结果说明-3)
    - [get_sudt_balance](#get_sudt_balance)
      - [输入参数说明](#输入参数说明-4)
      - [结果说明](#结果说明-4)
    - [get_crosschain_history](#get_crosschain_history)
      - [输入参数说明](#输入参数说明-5)
      - [结果说明](#结果说明-5)
    - [get_eth_to_ckb_status](#get_eth_to_ckb_status)
      - [输入参数说明](#输入参数说明-6)
      - [结果说明](#结果说明-6)    
    - [get_ckb_to_eth_status](#get_ckb_to_eth_status)
      - [输入参数说明](#输入参数说明-6)
      - [结果说明](#结果说明-6)


## API 接口文档

### settings

- path: `/settings`
- method: get

> 后文中的标题即为 path，方法默认为 post

用户连接到 dex 时，获取 force bridge settings。前端可以保存该配置备用。

```json
{
    "bridge_lockscript": {
        "code_hash": "fd9515dc15ce2385aab85af21a6c89d7c003eac115dcbd195a8f29ad916ab316",
        "hash_type": 1,
        "outpoint": {
            "dep_type": 0,
            "index": 0,
            "tx_hash": "23f30b8479415e5813e019b6b1783464f9c45dc5af333decda9650f90b0e3107"
        }
    },
    "bridge_typescript": {
        "code_hash": "a878bee27cf7fae49a028cf3e506af946dd2ea86f19586d68db3029ab3f20dd3",
        "hash_type": 1,
        "outpoint": {
            "dep_type": 0,
            "index": 1,
            "tx_hash": "23f30b8479415e5813e019b6b1783464f9c45dc5af333decda9650f90b0e3107"
        }
    },
    "ckb_relay_mutlisig_threshold": {
        "threshold": 0
    },
    "eth_ckb_chain_addr": "0xb01e8fd9657cCf5c4BE4fb4b1D665E1a3a491c1E",
    "eth_token_locker_addr": "0xF264A2Adf7D5c683855828B5bE39c25CEe0a13df",
    "light_client_cell_script": {
        "cell_script": "590000001000000030000000310000003130dc7bbf8b9e00ca9f7e9040bb59d242ee48375a322621be36e1f502a227ed0124000000ce121a960ca47b8cea3a9b3ddc75cb03e07c894c10d5557f865b50ddc6d68c8d01000000"
    },
    "light_client_typescript": {
        "code_hash": "3130dc7bbf8b9e00ca9f7e9040bb59d242ee48375a322621be36e1f502a227ed",
        "hash_type": 1,
        "outpoint": {
            "dep_type": 0,
            "index": 3,
            "tx_hash": "23f30b8479415e5813e019b6b1783464f9c45dc5af333decda9650f90b0e3107"
        }
    },
    "multisig_address": {
        "addresses": [
            "ckt1qyqyr27ps67cwn9cldgzvvmsa0lvry2wu5us4g430u"
        ],
        "require_first_n": 0,
        "threshold": 1
    },
    "pw_locks": {
        "inner": [
            {
                "dep_type": 0,
                "index": 0,
                "tx_hash": "57a62003daeab9d54aa29b944fc3b451213a5ebdf2e232216a3cfed0dde61b38"
            },
            {
                "dep_type": 1,
                "index": 0,
                "tx_hash": "f8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37"
            }
        ]
    },
    "recipient_typescript": {
        "code_hash": "ceb3993bd660eec67ab045f7853dd60b4b9d4e006db7069f7aae0e2e7a5037a5",
        "hash_type": 1,
        "outpoint": {
            "dep_type": 0,
            "index": 2,
            "tx_hash": "23f30b8479415e5813e019b6b1783464f9c45dc5af333decda9650f90b0e3107"
        }
    },
    "sudt": {
        "code_hash": "c5e5dcf215925f7ef4dfaf5f4b4f105bc321c02776d6e7d52a1db3fcd9d011a4",
        "hash_type": 1,
        "outpoint": {
            "dep_type": 0,
            "index": 0,
            "tx_hash": "e12877ebd2c3c364dc46c5c992bcfaf4fee33fa13eebdf82c591fc9825aab769"
        }
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
- eth_token_address：跨链 erc20 token 地址。eth 为 0000000000000000000000000000000000000000。


#### 结果说明
- 成功返回 200 code
- 重复 init 失败返回 400 code
```
// 重复 init 失败 的 response body
bad request data: token already inited
```


### lock

用户锁定资产到以太坊跨链合约。

```json
{
  "token_address": "0x3E35617a629EEaD5a6767dC69D238831a7Bc391c",
  "amount": "0x100",
  "bridge_fee": "0x1",
  "sender": "0x7ad9ec46a9c2910b446148728aced0c7e2b50048",
  "ckb_recipient_address": "ckt1qyqdtmf8yy3agyarxpwre60ydrl8082n5hysd4nn86",
  "sudt_extra_data": "bbc62a3943831e918f864a0306bc794b9e2724afc141b69d47e070f1f5686d4200000000",
  "gas_price": "0x1",
  "nonce": "0x1"
}
```

#### 输入参数说明
- token_address：要跨链的 token 地址。以太坊为 0x0000000000000000000000000000000000000000。
- amount：要跨链的数量，最大为 u128，hex 格式。
- bridge_fee：跨链手续费，从要跨的币中扣除。最大不超过 amount, hex 格式。
- ckb_recipient_address：ckb 接收跨链资产地址。
- sudt_extra_data：生成的 sudt 中的额外数据，hex bytes。
- gas_price：发送 lock 交易的 gas 费用， hex 格式。
- nonce：交易发起地址的 nonce，hex 格式。
- sender: 发送该交易的地址，用来估计 gas 费。为用户钱包的以太坊地址。

#### 结果说明

```json
{
    "data": "04a56b310000000000000000000000003e35617a629eead5a6767dc69d238831a7bc391c0000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000049490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000d5ed272123d413a3305c3ce9e468fe779d53a5c900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000024bbc62a3943831e918f864a0306bc794b9e2724afc141b69d47e070f1f5686d4200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000024bbc62a3943831e918f864a0306bc794b9e2724afc141b69d47e070f1f5686d420000000000000000000000000000000000000000000000000000000000000000",
    "gas": "0x2dc6c0",
    "gas_price": "0x1",
    "nonce": "0x1",
    "raw": "0101832dc6c0940592aa9fd1ce50636c501e1e2db8688466acc1ea80b9020404a56b310000000000000000000000003e35617a629eead5a6767dc69d238831a7bc391c0000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000049490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000d5ed272123d413a3305c3ce9e468fe779d53a5c900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000024bbc62a3943831e918f864a0306bc794b9e2724afc141b69d47e070f1f5686d4200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000024bbc62a3943831e918f864a0306bc794b9e2724afc141b69d47e070f1f5686d420000000000000000000000000000000000000000000000000000000000000000",
    "to": "0x0592aa9fd1ce50636c501e1e2db8688466acc1ea",
    "value": "0x0"
}
```

- raw 为以太坊未签名原始交易
- 其它字段为人类可读的以太坊交易字段

#### 前端自行组装交易指南

```javascript
let lockTokenABI = {
    "constant": false,
    "inputs": [
      {
        "name": "token",
        "type": "address"
      },
      {
        "name": "amount",
        "type": "uint256"
      },
      {
        "name": "bridgeFee",
        "type": "uint256"
      },
      {
        "name": "recipientLockscript",
        "type": "bytes"
      },
      {
        "name": "replayResistOutpoint",
        "type": "bytes"
      },
      {
        "name": "sudtExtraData",
        "type": "bytes"
      }
    ],
    "name": "lockToken",
    "outputs": [],
    "payable": false,
    "stateMutability": "nonpayable",
    "type": "function"
  };
let lockEthABI = {
    "constant": false,
    "inputs": [
      {
        "name": "bridgeFee",
        "type": "uint256"
      },
      {
        "name": "recipientLockscript",
        "type": "bytes"
      },
      {
        "name": "replayResistOutpoint",
        "type": "bytes"
      },
      {
        "name": "sudtExtraData",
        "type": "bytes"
      }
    ],
    "name": "lockETH",
    "outputs": [],
    "payable": true,
    "stateMutability": "payable",
    "type": "function"
  };

let lockTokenData = web3.eth.abi.encodeFunctionCall(lockTokenABI, ['0x3E35617a629EEaD5a6767dC69D238831a7Bc391c', '0x100', '0x1', '0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000d5ed272123d413a3305c3ce9e468fe779d53a5c9', '0xbbc62a3943831e918f864a0306bc794b9e2724afc141b69d47e070f1f5686d4200000000', '0xbbc62a3943831e918f864a0306bc794b9e2724afc141b69d47e070f1f5686d4200000000'])

// lockToken 的 6 个参数分别为:
// - erc20 token address
// - amount
// - bridgeFee
// - recipientLockscript: 这个与 API 中不同，API 中传入的是 address，自行组装需要把 Address decode 成 Script，传入 Script 的 hex 格式
// - replayResistOutpoint
// - sudtExtraData


let LockEthData = web3.eth.abi.encodeFunctionCall(lockEthABI, ['0x1', '0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000d5ed272123d413a3305c3ce9e468fe779d53a5c9', '0xbbc62a3943831e918f864a0306bc794b9e2724afc141b69d47e070f1f5686d4200000000', '0xbbc62a3943831e918f864a0306bc794b9e2724afc141b69d47e070f1f5686d4200000000'])

// lockETH 的 4 个参数分别为：
// - bridgeFee
// - recipientLockscript
// - replayResistOutpoint
// - sudtExtraData

// 另外，lockETH 的 amount 参数需要在 tx.value 中体现
```

- 前端调试时，可以调用该接口，比对 data 和自行组装的 data 是否一致
- 和以太坊交互时，`tx.to` 为 `/settings` 接口中返回的 `eth_token_locker_addr` 字段

### burn

用户在 ckb 上 burn 掉 mirror token，跨链回 eth。

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
        "cell_deps": [
            {
                "dep_type": "code",
                "out_point": {
                    "index": "0x0",
                    "tx_hash": "0x54fec0286bda25ca198ac8a192a5463c03f862fe39e70e805eb57c5f199512d7"
                }
            },
            {
                "dep_type": "code",
                "out_point": {
                    "index": "0x4",
                    "tx_hash": "0x54fec0286bda25ca198ac8a192a5463c03f862fe39e70e805eb57c5f199512d7"
                }
            },
            {
                "dep_type": "code",
                "out_point": {
                    "index": "0x5",
                    "tx_hash": "0x54fec0286bda25ca198ac8a192a5463c03f862fe39e70e805eb57c5f199512d7"
                }
            },
            {
                "dep_type": "dep_group",
                "out_point": {
                    "index": "0x0",
                    "tx_hash": "0xace5ea83c478bb866edf122ff862085789158f5cbff155b7bb5f13058555b708"
                }
            }
        ],
        "hash": "0x6e455e0cfc51cce120169f830696460ef15805a5d601e59d72139753d114a286",
        "header_deps": [],
        "inputs": [
            {
                "previous_output": {
                    "index": "0x8",
                    "tx_hash": "0xa563884b3686078ec7e7677a5f86449b15cf2693f3c1241766c6996f206cc541"
                },
                "since": "0x0"
            }
        ],
        "outputs": [
            {
                "capacity": "0x513619a00",
                "lock": {
                    "args": "0x470dcdc5e44064909650113a274b3b36aecb6dc7",
                    "code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
                    "hash_type": "type"
                },
                "type": {
                    "args": "0x",
                    "code_hash": "0xa170baee8a38fcc33a83a51db412a51b74101e931f7f90586de1971b11154ad4",
                    "hash_type": "data"
                }
            },
            {
                "capacity": "0x736f62d68bc40c0",
                "lock": {
                    "args": "0x470dcdc5e44064909650113a274b3b36aecb6dc7",
                    "code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
                    "hash_type": "type"
                },
                "type": null
            }
        ],
        "outputs_data": [
            "0x728060ab319e06ac6b5d5c0c74009f7d1f5730db00000000000000000000000000000000000000000592aa9fd1ce50636c501e1e2db8688466acc1eaf063a61a307ef090ff41f649ba1975860a24f0a500d0ad7cbf576d2d5ce202b80000000000000000000000000000000000000000000000000000000000000000",
            "0x"
        ],
        "version": "0x0",
        "witnesses": []
    }
}
```

#### 结果说明
- raw_tx: ckb 原始交易。格式与 ckb rpc 接口一致。用户签名后可以自行发交易。

### get_sudt_balance

获取 sudt 余额

```json
{
  "address": "ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4",
  "token_address": "0x0000000000000000000000000000000000000000"
}
```

#### 输入参数说明
- token_address：erc20 地址
- address：ckb 地址

```json
{
    "balance": "0x0"
}
```

#### 结果说明
- balance：用户 token_address erc20 在 ckb 上的 mirror token 余额

### get_crosschain_history


#### 输入参数说明

```json
{
    "eth_recipient_addr": "0x53ed257c0be400230ffec1121b14ff4b843ea3ec",
    "ckb_recipient_lockscript_addr": "ckt1q3vvtay34wndv9nckl8hah6fzzcltcqwcrx79apwp2a5lkd07fdxx7kea3r2ns53pdzxzjrj3t8dp3lzk5qysh9z9we"
}
```

- ckb_recipient_lockscript_addr: ckb 接收者的 lockscript address 地址
- eth_recipient_addr: 以太坊接收地址

#### 结果说明

```json
{
    "eth_to_ckb": [
        {
            "id": 2,
            "eth_tx_hash": "0x6a643547b65df16edddba6a785d4f28481e70b96e41b0b7e140fa4a0a84cd4d5",
            "ckb_tx_hash": "0x1c3162b980fdc20fe3e9c7211c3b56c5f6991789c526678eacdc05c10b953463",
            "status": "success",
            "sort": "eth_to_ckb",
            "amount": "0x174e4905ba000",
            "token_addr": "0x0000000000000000000000000000000000000000"
        }
    ],
    "ckb_to_eth": [
        {
            "id": 1,
            "eth_tx_hash": "0x54b7479a0fafcef3ca72d35a2e26e0e46db8a2f78c24de1d13b8b3c049587bdd",
            "ckb_tx_hash": "0x435af82bfcf6b3e445db2e0c2da9d08e61e7371390da0a357c98d72128fdf489",
            "status": "success",
            "sort": "ckb_to_eth",
            "amount": "0x17e5ea8c1a800",
            "token_addr": "0x0000000000000000000000000000000000000000"
        }
    ]
}
```

- token_addr: 跨链涉及的币种
- sort: 类型为 ckb_to_eth 或者 eth_to_ckb
- status:
    - success
    - error
    - pending
- amount: 跨链金额，hex 格式
- eth_tx_hash: 跨链挂单的以太坊锁定资产交易 hash
- ckb_tx_hash: 跨链挂单的 ckb 交易 hash

### get_eth_to_ckb_status

通过 eth_lock_tx_hash 获取跨链状态

#### 输入参数说明

```json
{
    "eth_lock_tx_hash": "0x85479dfc42878b74436f4ebe630be41ecf06f219d3e3dad92de48c89eb369758"
}
```

- eth_lock_tx_hash: 以太坊锁定交易的签名

#### 结果说明

```json
{
    "id": 176,
    "eth_lock_tx_hash": "0x85479dfc42878b74436f4ebe630be41ecf06f219d3e3dad92de48c89eb369758",
    "status": "error",
    "token_addr": null,
    "sender_addr": null,
    "locked_amount": null,
    "bridge_fee": null,
    "ckb_recipient_lockscript": null,
    "sudt_extra_data": null,
    "ckb_tx_hash": null,
    "err_msg": "Failed to generate eth proof for lock tx:0x85479dfc42878b74436f4ebe630be41ecf06f219d3e3dad92de48c89eb369758, after retry 3 times"
}
```

- status:
    - success
    - irreparable error
    - pending
- err_msg: 报错信息

### get_ckb_to_eth_status

通过 ckb_burn_tx_hash 获取跨链状态

#### 输入参数说明

```json
{
    "ckb_burn_tx_hash": "85479dfc42878b74436f4ebe630be41ecf06f219d3e3dad92de48c89eb369758"
}
```

- ckb_burn_tx_hash: ckb burn 交易的签名

#### 结果说明

```json
{
    "id": 176,
    "eth_lock_tx_hash": "0x85479dfc42878b74436f4ebe630be41ecf06f219d3e3dad92de48c89eb369758",
    "status": "error",
    "token_addr": null,
    "sender_addr": null,
    "locked_amount": null,
    "bridge_fee": null,
    "ckb_recipient_lockscript": null,
    "sudt_extra_data": null,
    "ckb_tx_hash": null,
    "err_msg": ""
}
```

- status:
    - success
    - pending
- err_msg: ""

