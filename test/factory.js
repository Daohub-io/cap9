const debug = require('debug')
const assert = require('assert')

const Factory = artifacts.require('./Factory.sol')

const testutils = require("./testutils.js");

// Valid Contracts
const Valid =  {
    Adder: artifacts.require('test/valid/Adder.sol'),
    SysCallTestWrite: artifacts.require('test/valid/SysCallTestWrite.sol'),
    SysCallTestCall: artifacts.require('test/valid/SysCallTestCall.sol'),
    SysCallTestLog: artifacts.require('test/valid/SysCallTestLog.sol'),
}

const Invalid = {
    Call: artifacts.require('test/invalid/Call'),
    Callcode: artifacts.require('test/invalid/Callcode'),
    Delegatecall: artifacts.require('test/invalid/Delegatecall'),
    Create: artifacts.require('test/invalid/Create'),
    Suicide: artifacts.require('test/invalid/Suicide'),
}

const TestWrite = artifacts.require('test/TestWrite.sol');

function isNullAddress(address) {
    return address === "0x0000000000000000000000000000000000000000";
}

const testDebug = debug('test:Factory')
let testAccount = 0;

contract('Factory', function (accounts) {

    describe('.validate()', async function() {

        it('should accept valid contract', async function () {
            let factory = await Factory.new();
            let result = await factory.validate(testutils.trimSwarm(Valid.Adder.bytecode), {from: accounts[0]});
            const tx = await factory.validate.sendTransaction(testutils.trimSwarm(Valid.Adder.bytecode), {from: accounts[0]});
            const receipt = web3.eth.getTransactionReceipt(tx);
            console.log(`receipt.gasUsed: ${receipt.gasUsed}`);
            assert.equal(0, result.toNumber());
        })

        it('should reject a contract if it uses CREATE', async function () {
            let factory = await Factory.new();
            let result = await factory.validate(testutils.trimSwarm(Invalid.Create.bytecode), {from: accounts[0]});
            assert.equal(8, result.toNumber());
        })

        it('should reject a contract if it uses CALL', async function () {
            let factory = await Factory.new();
            let result = await factory.validate(testutils.trimSwarm(Invalid.Call.bytecode), {from: accounts[0]});
            assert.equal(9, result.toNumber());
        })

        it('should reject a contract if it uses CALLCODE', async function () {
            let factory = await Factory.new();
            let result = await factory.validate(testutils.trimSwarm(Invalid.Callcode.bytecode), {from: accounts[0]});
            assert.equal(10, result.toNumber());
        })

        it('should reject a contract if it uses DELEGATECALL', async function () {
            let factory = await Factory.new();
            let result = await factory.validate(testutils.trimSwarm(Invalid.Delegatecall.bytecode), {from: accounts[0]});
            assert.equal(11, result.toNumber());
        })


        it('should reject a contract if it uses SELFDESTRUCT', async function () {
            let factory = await Factory.new();
            let result = await factory.validate(testutils.trimSwarm(Invalid.Suicide.bytecode), {from: accounts[0]});
            assert.equal(13, result.toNumber());
        })
    })
    describe('.validateContract()', async function() {

        it('should accept valid contract', async function () {
            let factory = await Factory.new();
            const deployedContract = await testutils.deployedTrimmed(Valid.Adder);
            let result = await factory.validateContract(deployedContract.address, {from: accounts[0]});
            assert.equal(0, result.toNumber());
        })

        it('should accept valid contract with system calls (write)', async function () {
            let factory = await Factory.new();
            const deployedContract = await testutils.deployedTrimmed(Valid.SysCallTestWrite);
            let result = await factory.validateContract(deployedContract.address, {from: accounts[0]});
            assert.equal(result.toNumber(), 0);
        })

        it('should accept valid contract with system calls (call)', async function () {
            let factory = await Factory.new();
            const deployedContract = await testutils.deployedTrimmed(Valid.SysCallTestCall);
            let result = await factory.validateContract(deployedContract.address, {from: accounts[0]});
            const tx = await factory.validateContract.sendTransaction(deployedContract.address, {from: accounts[0]});
            const receipt = web3.eth.getTransactionReceipt(tx);
            console.log(`receipt.gasUsed: ${receipt.gasUsed}`);
            assert.equal(result.toNumber(), 0);
        })

        it('should accept valid contract with system calls (log)', async function () {
            let factory = await Factory.new();
            const deployedContract = await testutils.deployedTrimmed(Valid.SysCallTestLog);
            let result = await factory.validateContract(deployedContract.address, {from: accounts[0]});
            assert.equal(result.toNumber(), 0);
        })

        it('should reject a contract if it uses CREATE', async function () {
            let factory = await Factory.new();
            const deployedContract = await testutils.deployedTrimmed(Invalid.Create);
            let result = await factory.validateContract(deployedContract.address, {from: accounts[0]});
            assert.equal(8, result.toNumber());
        })

        it('should reject a contract if it uses CALL', async function () {
            let factory = await Factory.new();
            const deployedContract = await testutils.deployedTrimmed(Invalid.Call);
            let result = await factory.validateContract(deployedContract.address, {from: accounts[0]});
            assert.equal(9, result.toNumber());
        })

        it('should reject a contract if it uses CALLCODE', async function () {
            let factory = await Factory.new();
            const deployedContract = await testutils.deployedTrimmed(Invalid.Callcode);
            let result = await factory.validateContract(deployedContract.address, {from: accounts[0]});
            assert.equal(10, result.toNumber());
        })

        it('should reject a contract if it uses DELEGATECALL', async function () {
            let factory = await Factory.new();
            const deployedContract = await testutils.deployedTrimmed(Invalid.Delegatecall);
            let result = await factory.validateContract(deployedContract.address, {from: accounts[0]});
            assert.equal(11, result.toNumber());
        })


        it('should reject a contract if it uses SELFDESTRUCT', async function () {
            let factory = await Factory.new();
            const deployedContract = await testutils.deployedTrimmed(Invalid.Suicide);
            let result = await factory.validateContract(deployedContract.address, {from: accounts[0]});
            assert.equal(13, result.toNumber());
        })

        it('should reject a contract if it uses SSTORE', async function () {
            let factory = await Factory.new();
            const deployedContract = await testutils.deployedTrimmed(TestWrite)
            let result = await factory.validateContract(deployedContract.address, {from: accounts[0]});
            assert.equal(2, result.toNumber());
        })
    })
})
