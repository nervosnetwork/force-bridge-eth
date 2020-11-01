pragma solidity ^0.5.10;

contract IHeaderVerifier {
    function verifyHeader(bytes calldata input) external returns (bool);
}
