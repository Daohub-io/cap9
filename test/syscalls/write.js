const debug = require('debug')
const assert = require('assert')

const Kernel = artifacts.require('./Kernel.sol')
const abi = require('ethereumjs-abi')

const beakerlib = require("../../beakerlib");

// Valid Contracts
const Valid = {
    Adder: artifacts.require('test/valid/Adder.sol'),
    Multiply: artifacts.require('test/valid/Multiply.sol'),
    Divide: artifacts.require('test/valid/Divide.sol'),
    SysCallTest: artifacts.require('test/valid/SysCallTest.sol'),
    SysCallTestLog: artifacts.require('test/valid/SysCallTestLog.sol'),
}

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

contract('Kernel', function (accounts) {
    describe('Write SysCall Procedure', function () {
        it('S() should succeed when given cap', async function () {
            const kernel = await Kernel.new();

            const cap1 = new beakerlib.WriteCap(0x8500,2);
            const cap2 = new beakerlib.WriteCap(0x8000,0);
            const capArray = beakerlib.Cap.toInput([cap1, cap2]);

            const tx1 = await kernel.createProcedure("SysCallTest", Valid.SysCallTest.bytecode, capArray);
            const tx2 = await kernel.createProcedure("Simple", Invalid.Simple.bytecode, []);

            const newValue1 = await kernel.testGetter.call();
            assert.equal(newValue1.toNumber(), 3, "The value should be 3 before the first execution");

            const valueX = await kernel.executeProcedure.call("SysCallTest", "S()", "");
            await kernel.executeProcedure("SysCallTest", "S()", "");
            assert.equal(valueX.toNumber(), 111111, "S() should succeed with correct value the first time");
            const newValue2 = await kernel.testGetter.call();
            assert.equal(newValue2.toNumber(), 4, "The value should be 4 after the first execution");

            // do it again to check that the value has been correctly incremented
            const value2 = await kernel.executeProcedure.call("SysCallTest", "S()", "");
            await kernel.executeProcedure("SysCallTest", "S()", "");
            assert.equal(value2.toNumber(), 111111, "S() should succeed with correct value the second time");
            const newValue3 = await kernel.testGetter.call();
            assert.equal(newValue3.toNumber(), 5, "The value should be 5 after the second execution");
        })
        it('S() should fail when not given cap', async function () {
            const kernel = await Kernel.new();
            const [, address] = await kernel.createProcedure.call("SysCallTest", Valid.SysCallTest.bytecode, []);
            const tx = await kernel.createProcedure("SysCallTest", Valid.SysCallTest.bytecode, []);

            const newValue1 = await kernel.testGetter.call();
            assert.equal(newValue1.toNumber(), 3, "The value should be 3 before the first execution");
            const valueX = await kernel.executeProcedure.call("SysCallTest", "S()", "");
            await kernel.executeProcedure("SysCallTest", "S()", "");
            assert.equal(valueX.toNumber(), 222222, "S() should succeed with correct value the first time");
            const newValue2 = await kernel.testGetter.call();
            assert.equal(newValue2.toNumber(), 3, "The value should still be 3 before the first execution");

            // do it again
            const value2 = await kernel.executeProcedure.call("SysCallTest", "S()", "");
            await kernel.executeProcedure("SysCallTest", "S()", "");
            assert.equal(value2.toNumber(), 222222, "S() should succeed with correct value the second time");
            const newValue3 = await kernel.testGetter.call();
            assert.equal(newValue3.toNumber(), 3, "The value should still be 3 before the second execution");
        })
        it('S() should fail when trying to write to an address below its cap', async function () {
            const kernel = await Kernel.new();
            const [, address] = await kernel.createProcedure.call("SysCallTest", Valid.SysCallTest.bytecode, [3, 0x7, 0x8001, 0x0]);
            const tx = await kernel.createProcedure("SysCallTest", Valid.SysCallTest.bytecode, [3, 0x7, 0x8001, 0x0]);

            const newValue1 = await kernel.testGetter.call();
            assert.equal(newValue1.toNumber(), 3, "The value should be 3 before the first execution");
            const valueX = await kernel.executeProcedure.call("SysCallTest", "S()", "");
            await kernel.executeProcedure("SysCallTest", "S()", "");
            assert.equal(valueX.toNumber(), 222222, "S() should fail with correct value the first time");
            const newValue2 = await kernel.testGetter.call();
            assert.equal(newValue2.toNumber(), 3, "The value should remain the same the first time");

            // do it again
            const value2 = await kernel.executeProcedure.call("SysCallTest", "S()", "");
            await kernel.executeProcedure("SysCallTest", "S()", "");
            assert.equal(value2.toNumber(), 222222, "S() should fail with correct value the second time");
            const newValue3 = await kernel.testGetter.call();
            assert.equal(newValue3.toNumber(), 3, "The value should remain the same the second time");
        })
    })
})