pragma solidity ^0.4.17;

contract Multiply {
    event LogFundsReceivedMultiply(address sender, uint amount);
    event LogFundsSent(address receiver, uint amount);

    function Multiply() payable public {
        LogFundsReceivedMultiply(msg.sender, msg.value);
    }
    function() payable public {
        LogFundsReceivedMultiply(msg.sender, msg.value);
    }

    function multiply(uint a, uint b) pure public returns (uint256) {
        return a * b;
    }
}