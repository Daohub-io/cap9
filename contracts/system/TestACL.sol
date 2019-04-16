pragma solidity ^0.4.17;

import "./ACL.sol";

contract TestACL is ACL {

    function getGroupByIndex(uint8 _groupIndex) public returns (bytes24 procId, uint8 accountsLen, uint8 groupIndex) {
        return _getGroupByIndex(_groupIndex);
    }

    function getAccountById(address _accountId) public returns (address accountId, bytes24 procId, uint8 accountIndex) {
        return _getAccountById(_accountId);
    }

    function getAccountByIndex(uint8 _accountIndex) public returns (address accountId, bytes24 procId, uint8 accountIndex) {
        return _getAccountByIndex(_accountIndex);
    }

    function createGroup(bytes24 _procId) public returns (uint8 groupIndex) {
        return _createGroup(_procId);
    }

    function removeGroup(bytes24 _procId) public returns (uint8 groupIndex) {
        return _removeGroup(_procId);
    }

    function addAccount(address _accountId, bytes24 _procId) public returns (uint8 accountIndex) {
        return _addAccount(_accountId, _procId);
    }

    function removeAccount(address _accountId) public {
        _removeAccount(_accountId);
    }

    
}