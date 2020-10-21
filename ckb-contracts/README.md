## ckb-contract 模块

### eth-light-client
> todo

### eth-bridge

#### 简介

将 ETH 和 ERC20 映射到 CKB 链上

#### 说明

- 资产 X：被映射的 ETH token 资产

- 资产 XT：X 映射到 CKB 上对应的资产

- token_address：ETH token 的合约地址
> 此处如果锁仓的是 ETH，则 token_address 的值为 0x0000000000000000000000000000000000000000

- eth_lock_address：ETH 上的 lock 合约地址

- eth_light_client：CKB 上维护的 ETH 轻节点，保存最近的 500 个 ETH headers

- receipt_proof：用户在 ETH 上向 eth_lock_address 锁定 token 后，使用这笔交易生成的 proof

#### 链上设计

mint xt 交易：

- 获取到 receipt_proof 后，用户发起 mint_xt 交易，生成 CKB 上的 XT

burn xt 交易：

- 用户销毁 CKB 上的 XT

#### Mint XT

获取到 receipt_proof 后，用户向 CKB 发送 Mint XT 交易，提供 spv proof 并且使用 eth_light_client

- cell deps

  - eth_light_client cell

- witess
   
  - spv proof
  
- inputs

  - any cell 

- outputs

  - token cell
    - capacity
        - 400 CKB (TOKEN_CAPACITY)
    - data
        - XT recipient lockscript hash：指定接收 XT 的地址
        - X token_address：token address
        - X address：ETH 地址
        - X token_amount: 锁仓 ERC20 token 数量
    - typescript
        - args 
            - token_address: ERC20 token address
            - cell_id: 第一个 input 的 outpoint
        - code_hash
            - inputs 中没有 ERC20 cell，output中有且只有一个
            - verify witness，校验 receipt proof 的合法性，确定 receipt proof 是 user 向 eth_lock_address 合约转移 token，且这笔交易在 eth_light_client 的块内。 
              解析出 合约地址 token_address，发送方地址 sender_address, 锁仓的 ERC20 数量以及 CKB 地址 ckb_address
            - verify sender address，判断 sender_address 等于 data 中的 address
            - verify token address，判断 token_address 等于 args 中地址
            - verify XT recipient address，校验 ckb_address 和 XT recipient lockscript hash
            - verify XT，校验 user 收到的 XT 数量和锁仓的 ERC20 数量一致，且等于 data 中的 XT amount
    - lockscript
        - code_hash
            - 校验 typescript == args 的任何 cell，它的 lockscript hash == self-hash
        - args 
            - token typescript hash      
  - XT cell
    - capacity
        - 200 CKB (XT_CAPACITY) 
    - data
        - ERC20 锁仓 amount
    - typescript
        - args
            - token lockscript hash

#### Burn XT

用户向 CKB 发起 Burn XT 交易，销毁 XT

- inputs

  - token cell
      - typescript
          - inputs 中有且只有一个 ERC20 cell，output中没有
          - verify token address，判断 XT cell 所对应的 token_address 等于 data 中 token_address
          - verify XT，销毁的 XT 数量和 data 的数量一致
          - verify XT receipient lockscript，判断 XT cell 的 lockscript 等于 data 中的 XT receipient lockscript
             
  - XT cell
 
- outputs
  
  - user cell
      - capacity
           - TOKEN_CAPACITY + XT_CAPACITY


## development

```bash
# install capsule 0.4.1
cargo install capsule --git https://github.com/nervosnetwork/capsule.git --tag 0.4.1

# test
make
```

