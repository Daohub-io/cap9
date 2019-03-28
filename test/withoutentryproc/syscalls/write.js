const debug = require('debug')
const assert = require('assert')

const Kernel = artifacts.require('./TestKernel.sol')
const abi = require('ethereumjs-abi')

const beakerlib = require("../../../beakerlib");
const testutils = require("../../testutils.js");

// Valid Contracts
const Valid = {
    Adder: artifacts.require('test/valid/Adder.sol'),
    Multiply: artifacts.require('test/valid/Multiply.sol'),
    Divide: artifacts.require('test/valid/Divide.sol'),
    SysCallTestWrite: artifacts.require('test/valid/SysCallTestWrite.sol'),
    Simple: artifacts.require('test/valid/Simple.sol'),
    SysCallTestLog: artifacts.require('test/valid/SysCallTestLog.sol'),
}

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

const SysCallResponse = beakerlib.SysCallResponse;

contract('Kernel without entry procedure', function (accounts) {
    describe('Write SysCall Procedure', function () {
        it('S() should succeed when given cap', async function () {
            const kernel = await Kernel.new();

            const cap1 = new beakerlib.WriteCap(0x8500,2);
            const cap2 = new beakerlib.WriteCap(0x8000,0);
            const capArray = beakerlib.Cap.toInput([cap1, cap2]);

            const SysCallTestWrite = await testutils.deployedTrimmed(Valid.SysCallTestWrite);
            const simpleTest = await testutils.deployedTrimmed(Valid.Multiply);
            const tx1 = await kernel.registerProcedure("SysCallTestWrite", SysCallTestWrite.address, capArray);
            const tx2 = await kernel.registerProcedure("Simple", simpleTest.address, []);

            const newValue1 = await kernel.testGetter.call();
            assert.equal(newValue1.toNumber(), 3, "The value should be 3 before the first execution");

            const valueX = await kernel.executeProcedure.call("SysCallTestWrite", "S()", "");
            await kernel.executeProcedure("SysCallTestWrite", "S()", "");
            assert.equal(valueX.toNumber(), SysCallResponse.SUCCESS, "S() should succeed with correct value the first time");
            const newValue2 = await kernel.testGetter.call();
            assert.equal(newValue2.toNumber(), 4, "The value should be 4 after the first execution");

            // do it again to check that the value has been correctly incremented
            const value2 = await kernel.executeProcedure.call("SysCallTestWrite", "S()", "");
            await kernel.executeProcedure("SysCallTestWrite", "S()", "");
            assert.equal(value2.toNumber(), SysCallResponse.SUCCESS, "S() should succeed with correct value the second time");
            const newValue3 = await kernel.testGetter.call();
            assert.equal(newValue3.toNumber(), 5, "The value should be 5 after the second execution");
        })
        it('S() should fail when not given cap', async function () {
            const kernel = await Kernel.new();
            const SysCallTestWrite = await testutils.deployedTrimmed(Valid.SysCallTestWrite);
            const [, address] = await kernel.registerProcedure.call("SysCallTestWrite", SysCallTestWrite.address, []);
            const tx = await kernel.registerProcedure("SysCallTestWrite", SysCallTestWrite.address, []);

            const newValue1 = await kernel.testGetter.call();
            assert.equal(newValue1.toNumber(), 3, "The value should be 3 before the first execution");
            const valueX = await kernel.executeProcedure.call("SysCallTestWrite", "S()", "");
            await kernel.executeProcedure("SysCallTestWrite", "S()", "");
            assert.equal(valueX.toNumber(), SysCallResponse.WRITEFAILURE, "S() should fail with correct value the first time");
            const newValue2 = await kernel.testGetter.call();
            assert.equal(newValue2.toNumber(), 3, "The value should still be 3 before the first execution");

            // do it again
            const value2 = await kernel.executeProcedure.call("SysCallTestWrite", "S()", "");
            await kernel.executeProcedure("SysCallTestWrite", "S()", "");
            assert.equal(value2.toNumber(), SysCallResponse.WRITEFAILURE, "S() should fail with correct value the second time");
            const newValue3 = await kernel.testGetter.call();
            assert.equal(newValue3.toNumber(), 3, "The value should still be 3 before the second execution");
        })
        it('S() should fail when trying to write to an address below its cap', async function () {
            const kernel = await Kernel.new();

            const SysCallTestWrite = await testutils.deployedTrimmed(Valid.SysCallTestWrite);

            const cap1 = new beakerlib.WriteCap(0x8001, 0);
            const capArray = beakerlib.Cap.toInput([cap1]);

            const [, address] = await kernel.registerProcedure.call("SysCallTestWrite", SysCallTestWrite.address, capArray);
            const tx = await kernel.registerProcedure("SysCallTestWrite", SysCallTestWrite.address, capArray);

            const newValue1 = await kernel.testGetter.call();
            assert.equal(newValue1.toNumber(), 3, "The value should be 3 before the first execution");
            const valueX = await kernel.executeProcedure.call("SysCallTestWrite", "S()", "");
            const txn = await kernel.executeProcedure("SysCallTestWrite", "S()", "");
            assert.equal(valueX.toNumber(), SysCallResponse.WRITEFAILURE, "S() should fail with correct value the first time");
            const newValue2 = await kernel.testGetter.call();
            assert.equal(newValue2.toNumber(), 3, "The value should remain the same the first time");

            // do it again
            const value2 = await kernel.executeProcedure.call("SysCallTestWrite", "S()", "");
            await kernel.executeProcedure("SysCallTestWrite", "S()", "");
            assert.equal(value2.toNumber(), SysCallResponse.WRITEFAILURE, "S() should fail with correct value the second time");
            const newValue3 = await kernel.testGetter.call();
            assert.equal(newValue3.toNumber(), 3, "The value should remain the same the second time");
        })
    })
})
