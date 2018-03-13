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

    describe('.create()', async function () {

        const Adder = Valid.Adder;
        const Multiply = Valid.Multiply;

        it('should be able to see the bytecode', async function () {
            const factory = await Factory.new();
            // Adder.bytecode is an array of bytes encoded as a string (hexadecimal)
            const rawBytes = web3.toAscii(Adder.bytecode);
            // facory.codeLength simply takes the array of bytes and counts them
            let codel = await factory.codeLength.call(Adder.bytecode, {from: accounts[testAccount]});
            assert.equal(rawBytes.length, codel.toNumber());
        })

        it('should use the correct code position', async function () {
            const factory = await Factory.new();
            // Peform an ephemeral call to Factory.create
            const codePos = await factory.codePosition.call(Adder.bytecode, {from: accounts[testAccount]});
            const codePosN = codePos.toNumber();
            assert(typeof codePosN === "number", "the code position is a number");
            assert(codePosN >= 0, "the code position is not negative");
        })

        it('should create a sample contract', async function () {
            const factory = await Factory.new();
            // Peform an ephemeral call to Factory.create
            let address = await factory.create.call(Adder.bytecode, {from: accounts[testAccount]});
            assert(web3.isAddress(address), `The returned address (${address}) is a valid address`);
            assert(!isNullAddress(address), `The returned address (${address}) is not the null address`);
        })

        it('the returned address should be deterministic and valid', async function () {
            const factory = await Factory.new();
            // Perform two ephemeral calls to factory.create
            let address1 = await factory.create.call(Adder.bytecode, {from: accounts[testAccount]});
            let address2 = await factory.create.call(Adder.bytecode, {from: accounts[testAccount]});
            // Addresses are the same.
            assert.equal(address1, address2);
            // The addresses are valid.
            assert(web3.isAddress(address1), `The returned address (${address1}) is a valid address`);
            assert(web3.isAddress(address2), `The returned address (${address2}) is a valid address`);
            // The addresses are not null.
            assert(!isNullAddress(address1), `The returned address (${address1}) is not the null address`);
            // const code = web3.eth.getCode(address1);
            // assert.equal(Adder.bytecode)
        })

        it('the returned address should not be deterministic if we make an additional transaction in between', async function () {
            const factory = await Factory.new();
            // Perform two ephemeral calls to factory.create
            const address1 = await factory.create.call(Adder.bytecode, {from: accounts[testAccount]});
            const tx = await factory.create(Adder.bytecode, {from: accounts[testAccount]});
            const address2 = await factory.create.call(Adder.bytecode, {from: accounts[testAccount]});
            // The addresses are valid.
            assert(web3.isAddress(address1), `The returned address (${address1}) is a valid address`);
            assert(web3.isAddress(address2), `The returned address (${address2}) is a valid address`);
            // The addresses are not null.
            assert(!isNullAddress(address1), `The returned address (${address1}) is not the null address`);
            // Addresses are different.
            assert.notEqual(address1, address2);
        })

        it('the new contract should function properly', async function () {
            const factory = await Factory.new();
            let address = await factory.create.call(Adder.bytecode, {from: accounts[testAccount]});
            let tx = await factory.create(Adder.bytecode, {from: accounts[testAccount]});
            assert(web3.isAddress(address), `The returned address (${address}) is a valid address`);
            assert(!isNullAddress(address), `The returned address (${address}) is not the null address`);

            let adder = Adder.at(address);
            let two = await adder.add.call(1, 1);
            assert.equal(two, 2);

            // The returned code should be the same as the sent code
            const code = web3.eth.getCode(address);
            assert.equal(Adder.deployedBytecode, code);
        })

    })

    describe('.validate()', async function() {

        it('should accept valid contract', async function () {
            let factory = await Factory.deployed();
            let result = await factory.validate(Valid.Adder.bytecode, {from: accounts[0]});
            assert.equal(0, result.toNumber());
        })
        
        it('should reject a contract if it uses CREATECALL', async function () {
            let factory = await Factory.deployed();
            let result = await factory.validate(Invalid.Create.bytecode, {from: accounts[0]});
            assert.equal(1, result.toNumber());
        })

        it('should reject a contract if it uses CALL', async function () {
            let factory = await Factory.deployed();
            let result = await factory.validate(Invalid.Call.bytecode, {from: accounts[0]});
            assert.equal(2, result.toNumber());
        })

        it('should reject a contract if it uses CALLCODE', async function () {
            let factory = await Factory.deployed();
            let result = await factory.validate(Invalid.Callcode.bytecode, {from: accounts[0]});
            assert.equal(3, result.toNumber());
        })

        it('should reject a contract if it uses DELEGATECALL', async function () {
            let factory = await Factory.deployed();
            let result = await factory.validate(Invalid.Delegatecall.bytecode, {from: accounts[0]});
            assert.equal(4, result.toNumber());
        })

        it('should reject a contract if it uses SUICIDECALL', async function () {
            let factory = await Factory.deployed();
            let result = await factory.validate(Invalid.Suicide.bytecode, {from: accounts[0]});
            assert.equal(5, result.toNumber());
        })
    })

})
