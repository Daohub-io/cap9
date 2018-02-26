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
        
        it('should create a sample contract', async function () {
            let factory = await Factory.deployed();
            let address = await factory.create(Adder.bytecode, {from: accounts[0]});
    
            let adder = Adder.at(address)
            let two = await Adder.add.call(1, 1);
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
