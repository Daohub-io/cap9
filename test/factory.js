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
    Delegatecall: artifacts.require('test/invalid/Delegatecall'),
    Create: artifacts.require('test/invalid/Create'),
    Suicide: artifacts.require('test/invalid/Suicide')
}

function isNullAddress(address) {
    return address === "0x0000000000000000000000000000000000000000";
}

const testDebug = debug('test:Factory')
let testAccount = 0;

contract('Factory', function (accounts) {

    describe('.validate()', async function() {

        it.skip('should accept valid contract', async function () {
            let factory = await Factory.deployed();
            let result = await factory.validate(Valid.Adder.bytecode, {from: accounts[0]});
            assert.equal(0, result.toNumber());
        })

        it('should reject a contract if it uses CREATE', async function () {
            let factory = await Factory.deployed();
            let result = await factory.validate(Invalid.Create.bytecode, {from: accounts[0]});
            assert.equal(8, result.toNumber());
        })

        it('should reject a contract if it uses CALL', async function () {
            let factory = await Factory.deployed();
            let result = await factory.validate(Invalid.Call.bytecode, {from: accounts[0]});
            assert.equal(9, result.toNumber());
        })

        it('should reject a contract if it uses CALLCODE', async function () {
            let factory = await Factory.deployed();
            let result = await factory.validate(Invalid.Callcode.bytecode, {from: accounts[0]});
            assert.equal(10, result.toNumber());
        })

        it('should reject a contract if it uses DELEGATECALL', async function () {
            let factory = await Factory.deployed();
            let result = await factory.validate(Invalid.Delegatecall.bytecode, {from: accounts[0]});
            assert.equal(11, result.toNumber());
        })


        it('should reject a contract if it uses SELFDESTRUCT', async function () {
            let factory = await Factory.deployed();
            let result = await factory.validate(Invalid.Suicide.bytecode, {from: accounts[0]});
            assert.equal(13, result.toNumber());
        })
    })
})
