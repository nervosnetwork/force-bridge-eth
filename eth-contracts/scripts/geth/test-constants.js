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
  const typeHash = await contract.callStatic.calcSetNewCkbSpvTypehash();

  const SET_NEW_CKB_SPV_TYPEHASH = keccak256(
    toUtf8Bytes('SetNewCkbSpv(address newSpvAddress,uint256 nonce)')
  );

  expect(typeHash).to.eq(SET_NEW_CKB_SPV_TYPEHASH);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
