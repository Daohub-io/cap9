const debug = require('debug')
const assert = require('assert')

const Factory = artifacts.require('./Factory.sol')

// Valid Contracts
const Valid =  {
    Adder: artifacts.require('test/valid/Adder.sol')
}

const Invalid = {
    Call: artifacts.require('test/invalid/Call'),
    Callcode: artifacts.require('test/invalid/Callcode'),
    Delegatecall: artifacts.require('test/invalid/Delegatecall')

}

// Test utility functions
function isNullAddress(address) {
    return address === "0x0000000000000000000000000000000000000000";
}

const testDebug = debug('test:Factory')
let testAccount = 0;

contract('ProcedureTable', function (accounts) {

    describe('.set()', async function () {

    })

    
})