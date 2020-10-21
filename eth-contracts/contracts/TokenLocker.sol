pragma solidity ^0.5.10;

import "./interfaces/IERC20.sol";
import {TypedMemView} from "./libraries/TypedMemView.sol";
import {SafeMath} from "./libraries/SafeMath.sol";
import {ViewCKB} from "./libraries/ViewCKB.sol";
import {ViewSpv} from "./libraries/ViewSpv.sol";
import {Address} from "./libraries/Address.sol";
import {ICKBSpv} from "./interfaces/ICKBSpv.sol";

contract TokenLocker {
    using SafeMath for uint256;
    using Address for address;

    using TypedMemView for bytes;
    using TypedMemView for bytes29;
    using ViewCKB for bytes29;
    using ViewSpv for bytes29;

    uint64 public numConfirmations_;
    ICKBSpv public ckbSpv_;

    // txHash -> Used
    mapping(bytes32 => bool) public usedTx_;

    event Locked(
        address indexed token,
        address indexed sender,
        uint256 amount,
        string accountId
    );

    event Unlocked(
        address indexed token,
        uint128 amount,
        address recipient
    );

    // Function output from burning fungible token on Near side.
    struct BurnResult {
        uint128 amount;
        address token;
        address recipient;
    }

    constructor(address ckbSpvAddress, uint64 numConfirmations) public {
        ckbSpv_ = ICKBSpv(ckbSpvAddress);
        numConfirmations_ = numConfirmations;
    }

    function _decodeBurnResult(bytes memory proofData) internal pure returns (BurnResult memory result) {
        // TODO
        // 1. verify burn tx
        // 2. _decodeBurnResult from proofData
        uint128 mockAmount = 111100000000000000;
        address mockToken = address(0);
        address mockRecipient = address(0);
        result = BurnResult(mockAmount, mockToken, mockRecipient);
    }

    // before lockToken, user should approve -> TokenLocker Contract with 0xffffff token
    function lockToken(address token, uint256 amount, string memory ckbAddress) public {
        // TODO modify `transferFrom` to `safeTransferFrom`
        IERC20(token).transferFrom(msg.sender, address(this), amount);
        emit Locked(token, msg.sender, amount, ckbAddress);
    }

    function lockETH(uint256 amount, string memory ckbAddress) public payable {
        emit Locked(address(0), msg.sender, msg.value, ckbAddress);
    }

    function unlockToken(bytes memory proofData) public {
        require(ckbSpv_.proveTxExist(proofData, numConfirmations_), "tx from proofData should exist");

        // Unpack the proof and extract the execution outcome.
        bytes29 proof = proofData.ref(uint40(ViewSpv.SpvTypes.TransactionProof));

        // TODO modify `mockTxHash` to `txHash`
        bytes32 txHash = proof.mockTxHash();
        require(!usedTx_[txHash], "The burn tx cannot be reused");
        usedTx_[txHash] = true;

        BurnResult memory result = _decodeBurnResult(proofData);
        // TODO modify `transfer` to `safeTransfer`
        if (result.token == address(0)) {
            // it means token == Eth
            result.recipient.toPayable().transfer(result.amount);
        } else {
            IERC20(result.token).transfer(result.recipient, result.amount);
        }

        emit Unlocked(address(result.token), result.amount, result.recipient);
    }

}
