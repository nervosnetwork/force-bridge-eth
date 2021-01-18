const fs = require('fs');
const TOML = require('@iarna/toml');
const EthUtil = require('ethereumjs-util');
const { keccak256, defaultAbiCoder, toUtf8Bytes } = ethers.utils;
const {
  getMockTinyHeaderParam,
  log,
  generateSignatures,
  getMsgHashForAddHeaders,
} = require('../test/utils');

// @dev set the latest tiny header to specific tiny header
// 1. require the contract factory path to use
// 2. require contract address
const contractFactoryPath = 'contracts/CKBChainV2-openzeppelin.sol:CKBChainV2';
const contractAddress = '0xe9B447cA594cB87B8d912040c8981B9696541B82';
const targetBlockNumber = 992353;
const targetBlockHash =
  'fe5a70bdf770a0c644b5f6fc9ea78a159c2447ed7bd56ae00f0a184a0245dcb3';

const main = async () => {
  log(`contractFactoryPath: `, contractFactoryPath);
  log(`contractAddress: `, contractAddress);

  const retryTimes = 1;
  for (let i = 0; i < retryTimes; i++) {
    try {
      await setSpecificTinyHeader(targetBlockNumber, targetBlockHash);
      log(`setSpecificTinyHeader success!`);
      break;
    } catch (e) {
      log(e);
      log('retry deploy times: ', i);
    }
  }
};

const setSpecificTinyHeader = async (blockNumber, blockHash) => {
  // get wallet of validators from force config
  const forceConfigPath = process.env.FORCE_CONFIG_PATH;
  const network = process.env.FORCE_NETWORK;
  if (!forceConfigPath) {
    throw 'FORCE_CONFIG_PATH not set';
  }
  const forceConfig = TOML.parse(fs.readFileSync(forceConfigPath));
  let network_config;
  if (network) {
    network_config = forceConfig.networks_config[network];
  } else {
    network_config = forceConfig.networks_config[forceConfig.default_network];
  }
  const provider = new ethers.providers.JsonRpcProvider(
    network_config.ethereum_rpc_url
  );
  const wallet = new ethers.Wallet(
    '0x' + network_config.ethereum_private_keys[0],
    provider
  );
  const adminAddress = wallet.address;
  console.error(`adminAddress : `, adminAddress);
  console.error(`adminAddress balance: `, await wallet.getBalance());

  const validators = network_config.ethereum_private_keys
    .slice(0, 2)
    .map((privateKey) => {
      let publicKey = EthUtil.privateToPublic(Buffer.from(privateKey, 'hex'));
      return '0x' + EthUtil.publicToAddress(publicKey).toString('hex');
    });
  console.error('validator validator: ', validators);
  const multisigThreshold = 1;
  let eth_network = await provider.getNetwork();
  const chainId = eth_network.chainId;
  console.error('chain id :', chainId);

  // 1. get mockTinyHeaders param
  const mockTinyHeaders = getMockTinyHeaderParam(blockNumber, blockHash);
  const name = 'Force Bridge CKBChain';
  const DOMAIN_SEPARATOR = keccak256(
    defaultAbiCoder.encode(
      ['bytes32', 'bytes32', 'bytes32', 'uint256', 'address'],
      [
        keccak256(
          toUtf8Bytes(
            'EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)'
          )
        ),
        keccak256(toUtf8Bytes(name)),
        keccak256(toUtf8Bytes('1')),
        chainId,
        '0xe9B447cA594cB87B8d912040c8981B9696541B82',
      ]
    )
  );
  const msgHash = getMsgHashForAddHeaders(
    DOMAIN_SEPARATOR,
    '0x1dac851def8ec317cf44b4a6cf63dabe82895259e6290d4c2ef271700bfce584',
    mockTinyHeaders
  );

  // 2. generate signatures
  const wallets = [wallet];
  let signatures = generateSignatures(
    msgHash,
    wallets.slice(0, multisigThreshold)
  );

  // 3. get instance
  const instance = await ethers.getContractAt(
    contractFactoryPath,
    contractAddress,
    wallet
  );

  // 4. call contract
  const tx = await instance.addHeaders(mockTinyHeaders, signatures);
  const receipt = await tx.wait(1);
  log(`receipt.status: `, receipt.status);
};

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
