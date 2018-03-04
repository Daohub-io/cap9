pragma solidity ^0.4.17;

contract Adder {
    event LogFundsReceivedAdder(address sender, uint amount);
    event LogFundsSent(address receiver, uint amount);

    function Adder() payable public {
        LogFundsReceivedAdder(msg.sender, msg.value);
    }
    function() payable public {
        LogFundsReceivedAdder(msg.sender, msg.value);
    }

    function add(uint a, uint b) pure public returns (uint256) {
        return a + b;
    }
}