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
    SysCallTestCall: artifacts.require('test/valid/SysCallTestCall.sol'),
    FirstNestedCall: artifacts.require('test/valid/NestedCalls/FirstNestedCall.sol'),
    SecondNestedCall: artifacts.require('test/valid/NestedCalls/SecondNestedCall.sol'),
    ThirdNestedCall: artifacts.require('test/valid/NestedCalls/ThirdNestedCall.sol'),
    FourthNestedCall: artifacts.require('test/valid/NestedCalls/FourthNestedCall.sol'),
    FifthNestedCall: artifacts.require('test/valid/NestedCalls/FifthNestedCall.sol'),
    SixthNestedCall: artifacts.require('test/valid/NestedCalls/SixthNestedCall.sol'),
}

const TestWrite = artifacts.require('test/TestWrite.sol');

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

contract('Kernel without entry procedure', function (accounts) {
    describe('Call capability', function () {
        const procName = "SysCallTestCall";
        const contract = Valid.SysCallTestCall;
        const bytecode = Valid.SysCallTestCall.bytecode;

        describe('A() - call procedure which needs no caps', function () {
            const testProcName = "TestWrite";
            const testBytecode = TestWrite.bytecode;
            const testContract = TestWrite;
            const functionSpec = "A()";
            it('A() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(0,"");
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                // console.log(web3.toHex(valueX))
                // try {
                //     console.log(web3.toAscii(web3.toHex(valueX)))
                // } catch (e) {

                // }
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                // console.log(tx)
                // for (const log of tx.receipt.logs) {
                //     if (log.topics.length > 0) {
                //         console.log(`Log: ${web3.toAscii(log.topics[0])} - ${log.data} - ${web3.toAscii(log.data)}`);
                //     } else {
                //         console.log(`Log: ${log.topics[0]} - ${web3.toAscii(log.data)} - ${log.data}`);
                //     }
                // }
                assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),356, "new value should be 356");
            })
            it('A() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(web3.toHex(valueX).slice(0,4), "0x55", "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
            it('A() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(web3.toHex(valueX).slice(0,4), "0x55", "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
            it('A() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(10, testProcName);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),356, "new value should be 356");
            })
            it('A() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(10, "another-proc");
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(web3.toHex(valueX).slice(0,4), "0x55", "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
        })
        describe('B() - without data', function () {
            const testProcName = "SysCallTestWrite";
            const testContract = Valid.SysCallTestWrite;
            const testBytecode = Valid.SysCallTestWrite.bytecode;
            const functionSpec = "B()";
            it('B() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(0,"");
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap2, cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),356, "new value should be 356");
            })
            it('B() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);


                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(web3.toHex(valueX).slice(0,4), "0x55", "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
            it('B() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(web3.toHex(valueX).slice(0,4), "0x55", "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
            it('B() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(10,testProcName);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap2, cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),356, "new value should be 356");
            })
            it('B() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(10,"another-proc");
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(web3.toHex(valueX).slice(0,4), "0x55", "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
        })
        describe('C() - with data (function selector)', function () {
            const testProcName = "SysCallTestWrite";
            const testBytecode = Valid.SysCallTestWrite.bytecode;
            const testContract = Valid.SysCallTestWrite;
            const functionSpec = "C()";
            it('C() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(0,"");
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap2, cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),(originalValue.toNumber() + 1), `new value should be ${originalValue.toNumber()+1}`);
            })
            it('C() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(web3.toHex(valueX).slice(0,4), "0x55", "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),originalValue.toNumber(), `new value should still be ${originalValue.toNumber()}`);
            })
            it('C() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(web3.toHex(valueX).slice(0,4), "0x55", "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),originalValue.toNumber(), `new value should still be ${originalValue.toNumber()}`);
            })
            it('C() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(10,testProcName);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap2, cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),(originalValue.toNumber() + 1), `new value should be ${originalValue.toNumber()+1}`);
            })
            it('C() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(10,"another-proc");
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(web3.toHex(valueX).slice(0,4), "0x55", "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),originalValue.toNumber(), `new value should still be ${originalValue.toNumber()}`);
            })
        })
        describe('E() - with data (function selector and arguments) and return', function () {
            const testProcName = "Adder";
            const testContract = Valid.Adder;
            const testBytecode = Valid.Adder.bytecode;
            const functionSpec = "E()";
            it('E() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(0,"");
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap2, cap1]));

                const newValue = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(newValue.toNumber(),8, `new value should be 8`);
            })
            it('E() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(web3.toHex(valueX).slice(0,4), "0x55", "should succeed with zero errcode the first time");
            })
            it('E() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(web3.toHex(valueX).slice(0,4), "0x55", "should succeed with zero errcode the first time");
            })
            it('E() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(10,testProcName);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap2, cap1]));

                const newValue = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(newValue.toNumber(),8, `new value should be 8`);
            })
            it('E() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(10,"another-proc");
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(web3.toHex(valueX).slice(0,4), "0x55", "should succeed with zero errcode the first time");
            })
        })
        describe('F() - successive calls single depth', function () {
            const testProcName = "Adder";
            const testBytecode = Valid.Adder.bytecode;
            const functionSpec = "F()";
            it('F() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(0,"");
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedAdderContract = await testutils.deployedTrimmed(Valid.Adder);
                const deployedSysCallTestContract = await testutils.deployedTrimmed(Valid.SysCallTestWrite);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the first called procedure, which doesn't really do anything
                await kernel.registerProcedure("Adder", deployedAdderContract.address, beakerlib.Cap.toInput([]));
                // // This is the second called procedure, which requires capabilities
                await kernel.registerProcedure("SysCallTestWrite", deployedSysCallTestContract.address, beakerlib.Cap.toInput([cap2, cap1]));

                const newValue = await kernel.executeProcedure.call(procName, functionSpec, "");
                // Execute
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                // console.log(tx);

                // console.log(tx.receipt.logs)
                // for (const log of tx.receipt.logs) {
                //     if (log.topics.length > 0) {
                //         console.log(`Log: ${web3.toAscii(log.topics[0])} - ${log.data} - ${web3.toAscii(log.data)}`);
                //     } else {
                //         console.log(`Log: ${log.topics[0]} - ${web3.toAscii(log.data)} - ${log.data}`);
                //     }
                // }
                // console.log(web3.toHex(newValue))
                assert.equal(newValue.toNumber(),8, `new value should be 8`);
                const newValue2 =  await kernel.testGetter.call();
                assert.equal(newValue2.toNumber(),4, "new value should be 4");
            })
        })
        describe('G() - deeper stacks', function () {
            const testProcName = "FirstNestedCall";
            const testBytecode = Valid.FirstNestedCall.bytecode;
            const functionSpec = "G()";
            it('G() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(0,"");
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedAdderContract = await testutils.deployedTrimmed(Valid.Adder);
                const deployedFirstNestedContract = await testutils.deployedTrimmed(Valid.FirstNestedCall);
                const deployedSecondNestedContract = await testutils.deployedTrimmed(Valid.SecondNestedCall);
                const deployedThirdNestedContract = await testutils.deployedTrimmed(Valid.ThirdNestedCall);
                const deployedFourthNestedContract = await testutils.deployedTrimmed(Valid.FourthNestedCall);
                const deployedFifthNestedContract = await testutils.deployedTrimmed(Valid.FifthNestedCall);
                const deployedSixthNestedContract = await testutils.deployedTrimmed(Valid.SixthNestedCall);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                await kernel.registerProcedure("Adder", deployedAdderContract.address, beakerlib.Cap.toInput([]));
                await kernel.registerProcedure("FirstNestedCall",  deployedFirstNestedContract.address,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8001,0), new beakerlib.CallCap(0,"")]));
                await kernel.registerProcedure("SecondNestedCall", deployedSecondNestedContract.address, beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8002,0), new beakerlib.CallCap(0,"")]));
                await kernel.registerProcedure("ThirdNestedCall",  deployedThirdNestedContract.address,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8003,0), new beakerlib.CallCap(0,"")]));
                await kernel.registerProcedure("FourthNestedCall", deployedFourthNestedContract.address, beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8004,0), new beakerlib.CallCap(0,"")]));
                await kernel.registerProcedure("FifthNestedCall",  deployedFifthNestedContract.address,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8005,0), new beakerlib.CallCap(0,"")]));
                await kernel.registerProcedure("SixthNestedCall",  deployedSixthNestedContract.address,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8006,0), new beakerlib.CallCap(0,"")]));

                await kernel.executeProcedure("FirstNestedCall", "G()", "");

                const firstVal = await kernel.anyTestGetter(0x8001);
                assert.equal(firstVal.toNumber(),75, `new value should be 75`);

                const secondVal = await kernel.anyTestGetter(0x8002);
                assert.equal(secondVal.toNumber(),76, `new value should be 76`);

                const thirdVal = await kernel.anyTestGetter(0x8003);
                assert.equal(thirdVal.toNumber(),77, `new value should be 77`);

                const fourthVal = await kernel.anyTestGetter(0x8004);
                assert.equal(fourthVal.toNumber(),78, `new value should be 78`);

                const fifthVal = await kernel.anyTestGetter(0x8005);
                assert.equal(fifthVal.toNumber(),79, `new value should be 79`);

                const sixthVal = await kernel.anyTestGetter(0x8006);
                assert.equal(sixthVal.toNumber(),80, `new value should be 80`);
            })
        })
    })
})
