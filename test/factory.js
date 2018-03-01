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

const testDebug = debug('test:Factory')

contract('Factory', function (accounts) {

    describe('.create()', async function () {

        const Adder = Valid.Adder;

        it('should be able to see the bytecode', async function () {
            let factory = await Factory.deployed();
            // Adder.bytecode is an array of bytes encoded as a string (hexadecimal)
            const rawBytes = web3.toAscii(Adder.bytecode);
            // facory.codeLength simply takes the array of bytes and counts them
            let codel = await factory.codeLength.call(Adder.bytecode, {from: accounts[0]});
            assert.equal(rawBytes.length, codel.toNumber());
        })

        it('should use the correct code position', async function () {
            const factory = await Factory.deployed();
            // Peform an ephemeral call to Factory.create
            const codePos = await factory.codePosition.call(Adder.bytecode, {from: accounts[0]});
            const codePosN = codePos.toNumber();
            assert(typeof codePosN === "number", "the code position is a number");
            assert(codePosN >= 0, "the code position is not negative");
        })

        it('should create a sample contract', async function () {
            let factory = await Factory.deployed();
            // Peform an ephemeral call to Factory.create
            let address = await factory.create.call(Adder.bytecode, {from: accounts[0]});
            assert(web3.isAddress(address), `The returned address (${address}) is a valid address`);
            assert(!isNullAddress(address), `The returned address (${address}) is not the null address`);
        })

        it('the returned address should be deterministic (and valid)', async function () {
            let factory = await Factory.deployed();
            let address1 = await factory.create.call(Adder.bytecode, {from: accounts[0]});
            let address2 = await factory.create.call(Adder.bytecode, {from: accounts[0]});
            assert.equal(address1, address2);
            let tx1 = await factory.create.estimateGas(Adder.bytecode, {from: accounts[0]});
            console.log(tx1);
            let tx = await factory.create(Adder.bytecode, {from: accounts[0], gas:tx1+10000});
            console.log(tx.receipt.gasUsed);
            assert(web3.isAddress(address1), `The returned address (${address1}) is a valid address`);
            assert(web3.isAddress(address2), `The returned address (${address2}) is a valid address`);
            assert(!isNullAddress(address1), `The returned address (${address1}) is not the null address`);
        })

        it('the new contract should function properly', async function () {
            let factory = await Factory.deployed();
            let address1 = await factory.create.call(Adder.bytecode, {from: accounts[0]});
            let address2 = await factory.create.call(Adder.bytecode, {from: accounts[0]});
            assert.equal(address1, address2);
            let tx = await factory.create(Adder.bytecode, {from: accounts[0]});
            assert(web3.isAddress(address1), `The returned address (${address1}) is a valid address`);
            assert(web3.isAddress(address2), `The returned address (${address2}) is a valid address`);

            let adder = Adder.at(address1);
            let two = await adder.add.call(1, 1);
            assert.equal(two, 2);
        })

    })

    describe('.validate()', async function() {

        it('should accept valid contract', async function () {
            let factory = await Factory.deployed();
            let valid = await factory.validate(Valid.Adder.bytecode, {from: accounts[0]});
            assert(valid);
        })

        it('should reject a contract if it uses CALL', async function () {
            let factory = await Factory.deployed();
            let valid = await factory.validate(Invalid.Call.bytecode, {from: accounts[0]});
            assert(!valid);
        })

        it('should reject a contract if it uses CALLCODE', async function () {
            let factory = await Factory.deployed();
            let valid = await factory.validate(Invalid.Callcode.bytecode, {from: accounts[0]});
            assert(!valid);
        })

        it('should reject a contract if it uses DELEGATECALL', async function () {
            let factory = await Factory.deployed();
            let valid = await factory.validate(Invalid.Delegatecall.bytecode, {from: accounts[0]});
            assert(!valid);
        })

        it('should reject a contract if it uses CREATE')
        it('should reject a contract if it uses SUICIDE')

    })

})

// Test utility functions
function isNullAddress(address) {
    return address === "0x0000000000000000000000000000000000000000";
}
