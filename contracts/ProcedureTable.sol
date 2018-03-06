pragma solidity ^0.4.17;

import "./Factory.sol";

contract ProcedureTable {

    function set(uint key, address procedure) public returns (bool error) {}

    function dispatch(uint key, bytes payload) public returns (bool error) {}

    function remove(uint key) public returns (bool error) {}

}