const { ecsign, toRpcSig } = require('ethereumjs-util')

async function sleep(seconds) {
  // console.log(`waiting for block confirmations, about ${seconds}s`)
  await new Promise((resolve) => setTimeout(resolve, seconds * 1000))
}

async function waitingForReceipt(provider, res) {
  if (!res) {
    return -1
  }

  const txHash = res.hash
  let txReceipt
  while (!txReceipt) {
    txReceipt = await provider.getTransactionReceipt(txHash)
    if (txReceipt && txReceipt.blockHash) {
      break
    }
    await sleep(1)
  }
  return txReceipt
}

const deployContract = async (factoryPath, ...args) => {
  // const factory = await ethers.getContractFactory("contracts/token/NFI.sol:NFI");
  const factory = await ethers.getContractFactory(factoryPath)
  const contract = await factory.deploy(...args)

  // await contract.deployed();
  await contract.deployTransaction.wait(1)
  return contract
}

const generateWallets = (size) => {
  const wallets = []
  for (let i = 0; i < size; i++) {
    const wallet = ethers.Wallet.createRandom()
    wallets.push(wallet)
  }
  return wallets
}

const generateSignatures = (msgHash, wallets) => {
  let signatures = '0x'
  for (let i = 0; i < wallets.length; i++) {
    const wallet = wallets[i]
    const { v, r, s } = ecsign(
      Buffer.from(msgHash.slice(2), 'hex'),
      Buffer.from(wallet.privateKey.slice(2), 'hex')
    )
    const sigHex = toRpcSig(v, r, s)
    signatures += sigHex.slice(2)
  }
  return signatures
}

const runErrorCase = async (txPromise, expectErrorMsg) => {
  try {
    await txPromise
  } catch (e) {
    const error = e.error ? e.error.toString() : e.toString()
    expect(error.indexOf(expectErrorMsg) > -1).to.eq(true)
  }
}

const { log } = console

module.exports = {
  sleep,
  log,
  waitingForReceipt,
  deployContract,
  generateWallets,
  generateSignatures,
  runErrorCase,
}
