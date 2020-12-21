// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import "./interfaces/IERC20.sol";
import {CKBCrypto} from "./libraries/CKBCrypto.sol";
import {TypedMemView} from "./libraries/TypedMemView.sol";
import {SafeMath} from "./libraries/SafeMath.sol";
import {CKBTxView} from "./libraries/CKBTxView.sol";
import {ViewSpv} from "./libraries/ViewSpv.sol";
import {Address} from "./libraries/Address.sol";
import {ICKBSpv} from "./interfaces/ICKBSpv.sol";
import {MultisigUtils} from "./libraries/MultisigUtils.sol";

contract TokenLocker {
    using SafeMath for uint256;
    using Address for address;
    using TypedMemView for bytes;
    using TypedMemView for bytes29;
    using CKBTxView for bytes29;
    using ViewSpv for bytes29;

    uint64 public numConfirmations_;
    ICKBSpv public ckbSpv_;
    bytes32 public recipientCellTypescriptCodeHash_;
    uint8 public recipientCellTypescriptHashType_;
    bytes32 public bridgeCellLockscriptCodeHash_;

    // txHash -> Used
    mapping(bytes32 => bool) public usedTx_;

    // refer to https://github.com/ethereum/EIPs/blob/master/EIPS/eip-712.md
    uint public constant SIGNATURE_SIZE = 65;
    uint public constant VALIDATORS_SIZE_LIMIT = 20;
    string public constant name = "Force Bridge TokenLocker";
    // SET_NEW_CKB_SPV_TYPEHASH = keccak256("SetNewCkbSpv(address newSpvAddress,uint256 nonce)");
    bytes32 public constant SET_NEW_CKB_SPV_TYPEHASH = 0x27ed7da1c419c8aa5f5c5f74f651895eaf8dab9831123b22248827a707d44bb1;
    bytes32 public DOMAIN_SEPARATOR;
    uint public currentSetNewCkbSpvNonce;
    // if the number of verified signatures has reached `multisigThreshold_`, validators approve the tx
    uint public multisigThreshold_;
    address[] validators_;

    event Locked(
        address indexed token,
        address indexed sender,
        uint256 lockedAmount,
        uint256 bridgeFee,
        bytes recipientLockscript,
        bytes replayResistOutpoint,
        bytes sudtExtraData
    );

    event Unlocked(
        address indexed token,
        address indexed recipient,
        address indexed sender,
        uint256 receivedAmount,
        uint256 bridgeFee
    );

    event NewCkbSpv(
        address ckbSpvAddress,
        uint256 nonce
    );

    constructor(
        address ckbSpvAddress,
        uint64 numConfirmations,
        bytes32 recipientCellTypescriptCodeHash,
        uint8 typescriptHashType,
        bytes32 bridgeCellLockscriptCodeHash,
        address[] memory validators,
        uint multisigThreshold,
        uint chainId
    ) {
        ckbSpv_ = ICKBSpv(ckbSpvAddress);
        numConfirmations_ = numConfirmations;
        recipientCellTypescriptCodeHash_ = recipientCellTypescriptCodeHash;
        recipientCellTypescriptHashType_ = typescriptHashType;
        bridgeCellLockscriptCodeHash_ = bridgeCellLockscriptCodeHash;

        // set DOMAIN_SEPARATOR
        DOMAIN_SEPARATOR = keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                keccak256(bytes(name)),
                keccak256(bytes("1")),
                chainId,
                address(this)
            )
        );

        // set validators
        require(validators.length <= VALIDATORS_SIZE_LIMIT, "number of validators exceeds the limit");
        validators_ = validators;
        require(multisigThreshold <= validators.length, "invalid multisigThreshold");
        multisigThreshold_ = multisigThreshold;
    }

    /**
     * @notice  if addr is not one of validators_, return validators_.length
     * @return  index of addr in validators_
     */
    function getIndexOfValidators(address user) internal view returns (uint) {
        for (uint i = 0; i < validators_.length; i++) {
            if (validators_[i] == user) {
                return i;
            }
        }
        return validators_.length;
    }

    /**
     * @notice             @dev signatures are a multiple of 65 bytes and are densely packed.
     * @param msgHash      sth. which signers sign
     * @param signatures   The signatures bytes array
     * @param threshold    check number of verified signatures >= `threshold`, signatures are approved by validators
     */
    function validatorsApprove(bytes32 msgHash, bytes memory signatures, uint threshold) public view {
        require(signatures.length % SIGNATURE_SIZE == 0, "invalid signatures");
        // 1. check length of signature
        uint length = signatures.length / SIGNATURE_SIZE;
        require(length >= threshold, "length of signatures must greater than threshold");

        // 2. check number of verified signatures >= threshold
        uint verifiedNum = 0;
        uint i = 0;

        uint8 v;
        bytes32 r;
        bytes32 s;
        address recoveredAddress;
        // set indexVisited[ index of recoveredAddress in validators_ ] = true
        bool[] memory validatorIndexVisited = new bool[](validators_.length);
        uint validatorIndex;
        while (i < length) {
            (v, r, s) = MultisigUtils.parseSignature(signatures, i);
            i++;

            recoveredAddress = ecrecover(msgHash, v, r, s);
            require(recoveredAddress != address(0), "invalid signature");

            // get index of recoveredAddress in validators_
            validatorIndex = getIndexOfValidators(recoveredAddress);

            // recoveredAddress is not validator or has been visited
            if (validatorIndex >= validators_.length || validatorIndexVisited[validatorIndex]) {
                continue;
            }

            // recoveredAddress verified
            validatorIndexVisited[validatorIndex] = true;
            verifiedNum++;
            if (verifiedNum >= threshold) {
                return;
            }
        }

        require(verifiedNum >= threshold, "signatures not verified");
    }

    function setNewCkbSpv(
        address newSpvAddress,
        uint nonce,
        bytes memory signatures
    ) public {
        // 1. check newSpvAddress and setNewCkbSpv nonce
        require(newSpvAddress != address(0), "invalid newSpvAddress");
        require(nonce == currentSetNewCkbSpvNonce, "invalid setNewCkbSpv nonce");
        currentSetNewCkbSpvNonce++;

        // 2. calc msgHash
        bytes32 msgHash = keccak256(
            abi.encodePacked(
                '\x19\x01', // solium-disable-line
                DOMAIN_SEPARATOR,
                keccak256(abi.encode(SET_NEW_CKB_SPV_TYPEHASH, newSpvAddress, nonce))
            )
        );

        // 3. check if validatorsApprove
        validatorsApprove(msgHash, signatures, multisigThreshold_);

        // 4. if validatorsApproved, set new ckbSpv_
        ckbSpv_ = ICKBSpv(newSpvAddress);

        emit NewCkbSpv(newSpvAddress, nonce);
    }

    function lockETH(
        uint256 bridgeFee,
        bytes memory recipientLockscript,
        bytes memory replayResistOutpoint,
        bytes memory sudtExtraData
    ) public payable {
        require(msg.value > bridgeFee, "fee should not exceed bridge amount");
        emit Locked(
            address(0),
            msg.sender,
            msg.value,
            bridgeFee,
            recipientLockscript,
            replayResistOutpoint,
            sudtExtraData
        );
    }

    // before lockToken, user should approve -> TokenLocker Contract with 0xffffff token
    function lockToken(
        address token,
        uint256 amount,
        uint256 bridgeFee,
        bytes memory recipientLockscript,
        bytes memory replayResistOutpoint,
        bytes memory sudtExtraData
    ) public {
        require(amount > bridgeFee, "fee should not exceed bridge amount");
        // TODO modify `transferFrom` to `safeTransferFrom`
        IERC20(token).transferFrom(msg.sender, address(this), amount);
        emit Locked(
            token,
            msg.sender,
            amount,
            bridgeFee,
            recipientLockscript,
            replayResistOutpoint,
            sudtExtraData
        );
    }

    function unlockToken(bytes memory ckbTxProof, bytes memory ckbTx) public {
        require(ckbSpv_.proveTxExist(ckbTxProof, numConfirmations_), "tx from proofData should exist");

        bytes29 proof = ckbTxProof.ref(uint40(ViewSpv.SpvTypes.CKBTxProof));
        bytes32 txHash = proof.txHash();
        require(!usedTx_[txHash], "The burn tx cannot be reused");
        usedTx_[txHash] = true;
        require((txHash == CKBCrypto.digest(ckbTx, ckbTx.length)), "ckbTx mismatched with ckbTxProof");

        (uint256 bridgeAmount, uint256 bridgeFee, address tokenAddress, address recipientAddress) = decodeBurnResult(ckbTx);
        require(bridgeAmount > bridgeFee, "fee should not exceed bridge amount");
        uint256 receivedAmount = bridgeAmount - bridgeFee;

        // TODO modify `transfer` to `safeTransfer`
        // if token == ETH
        if (tokenAddress == address(0)) {
            recipientAddress.toPayable().transfer(receivedAmount);
            payable(msg.sender).transfer(bridgeFee);
        } else {
            IERC20(tokenAddress).transfer(recipientAddress, receivedAmount);
            IERC20(tokenAddress).transfer(msg.sender, bridgeFee);
        }

        emit Unlocked(tokenAddress, recipientAddress, msg.sender, receivedAmount, bridgeFee);
    }

    // TODO refund function

    function decodeBurnResult(bytes memory ckbTx) public view returns (
        uint256 bridgeAmount,
        uint256 bridgeFee,
        address token,
        address recipient
    ){
        bytes29 rawTx = ckbTx.ref(uint40(CKBTxView.CKBTxTypes.RawTx));
        bytes29 recipientCellTypescript = rawTx.outputs().recipientCellOutput().typescript();
        require(
            (recipientCellTypescript.recipientTypescriptCodeHash() == recipientCellTypescriptCodeHash_),
            "invalid recipient cell typescript code hash"
        );
        require((recipientCellTypescript.hashType() == recipientCellTypescriptHashType_), "invalid recipient cell typescript hash type");
        bytes29 recipientCellData = rawTx.outputsData().recipientCellData();
        require((recipientCellData.contractAddress() == address(this)), "invalid contract address in recipient cell");
        require((recipientCellData.bridgeLockscriptCodeHash() == bridgeCellLockscriptCodeHash_), "invalid contract address in recipient cell");
        return (
            recipientCellData.bridgeAmount(),
            recipientCellData.bridgeFee(),
            recipientCellData.tokenAddress(),
            recipientCellData.recipientAddress()
        );
    }
}
