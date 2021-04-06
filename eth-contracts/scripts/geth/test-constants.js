const log = console.log.bind(console);
const { MaxUint256 } = ethers.constants;
const {
  bigNumberify,
  hexlify,
  keccak256,
  defaultAbiCoder,
  toUtf8Bytes,
} = ethers.utils;

async function main() {
  const factory = await ethers.getContractFactory(
    'contracts/test/TestConstants.sol:TestConstants'
  );
  const contract = await factory.deploy();
  await contract.deployed();
  const [
    history_typehash,
    validators_typehash,
  ] = await contract.callStatic.calcTypehash();

  const ADD_HISTORY_TX_ROOT_TYPEHASH = keccak256(
    toUtf8Bytes(
      'AddHistoryTxRoot(uint64 startBlockNumber, uint64 endBlockNumber, bytes32 historyTxRoot)'
    )
  );
  const SET_NEW_VALIDATORS_TYPEHASH = keccak256(
    toUtf8Bytes(
      'SetNewValidators(address[] validators, uint256 multisigThreshold)'
    )
  );

  log(`ADD_HISTORY_TX_ROOT_TYPEHASH: `, ADD_HISTORY_TX_ROOT_TYPEHASH);
  log(`SET_NEW_VALIDATORS_TYPEHASH: `, SET_NEW_VALIDATORS_TYPEHASH);

  expect(history_typehash).to.eq(ADD_HISTORY_TX_ROOT_TYPEHASH);
  expect(validators_typehash).to.eq(SET_NEW_VALIDATORS_TYPEHASH);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
