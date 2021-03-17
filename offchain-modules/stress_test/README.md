## 项目背景

压测脚本

## 项目流程

#### 1、 构造批量账户 burnPrivkeys。 以便并发执行 burn 操作
#### 2、RichCKB 账户批量转账CKB给这些账户，需要为 burn tx 提供 capacity
#### 3、RichETH 账户批量 lock token, 指定刚刚构造的 burnPrivkey 账户作为 recipient。
#### 4、等待 relay proof mint sudt 完成，并查询 burnPrivkeys sudt 账户余额
#### 5、burnPrivkeys 执行批量 burn 操作



## quick start

- 修改并发数量 `concurrency_number`和跨链的`cross_chain_tokens`:

`index.js` 文件中
```
   const concurrency_number = 2
   const cross_chain_tokens = ["ETH"]
```
需要确保`config.js` 文件中 `RichETHPrivkey` 和 `RichCKBPrivkey` 有钱


- cd dex-crosschain-bot && yarn : 下载依赖
- cd dex-crosschain-bot && yarn send 执行操作

