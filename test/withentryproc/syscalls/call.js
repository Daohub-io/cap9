const debug = require('debug')
const assert = require('assert')

const Kernel = artifacts.require('./Kernel.sol')
const abi = require('ethereumjs-abi')

const beakerlib = require("../../../beakerlib");
const testutils = require("../../testutils.js");

// Valid Contracts
const Valid = {
    Adder: artifacts.require('test/valid/Adder.sol'),
    Multiply: artifacts.require('test/valid/Multiply.sol'),
    Divide: artifacts.require('test/valid/Divide.sol'),
    SysCallTest: artifacts.require('test/valid/SysCallTest.sol'),
    SysCallTestCall: artifacts.require('test/valid/SysCallTestCall.sol'),
    FirstNestedCall: artifacts.require('test/valid/NestedCalls/FirstNestedCall.sol'),
    SecondNestedCall: artifacts.require('test/valid/NestedCalls/SecondNestedCall.sol'),
    ThirdNestedCall: artifacts.require('test/valid/NestedCalls/ThirdNestedCall.sol'),
    FourthNestedCall: artifacts.require('test/valid/NestedCalls/FourthNestedCall.sol'),
    FifthNestedCall: artifacts.require('test/valid/NestedCalls/FifthNestedCall.sol'),
    SixthNestedCall: artifacts.require('test/valid/NestedCalls/SixthNestedCall.sol'),
    BasicEntryProcedure: artifacts.require('BasicEntryProcedure.sol'),
}

const TestWrite = artifacts.require('test/TestWrite.sol');

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

contract('Kernel', function (accounts) {
    describe('Call capability', function () {
        const procName = "SysCallTestCall";
        const bytecode = Valid.SysCallTestCall.bytecode;

        describe('A() - call procedure which needs no caps', function () {
            const testProcName = "TestWrite";
            const testBytecode = TestWrite.bytecode;
            const functionSpec = "A()";
            it('A() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createAnyProcedure(testProcName, testBytecode, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");

                    const newValue =  await kernel.testGetter.call();
                    assert.equal(newValue.toNumber(),356, "new value should be 356");
                }

            })
            it('A() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                // This is the procedure that will do the calling, note that
                // it has no call cap
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createAnyProcedure(testProcName, testBytecode, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    for (const log of tx3.receipt.logs) {
                        if (log.topics.length > 0) {
                            console.log(`${web3.toAscii(log.topics[0])} - ${log.data} - ${web3.toAscii(log.data)}`);
                        } else {
                            console.log(`${log.topics[0]} - ${web3.toAscii(log.data)}`);
                        }
                    }

                    assert.equal(valueX.toNumber(), 4455, "should fail with correct errcode");
                }

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

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createAnyProcedure(testProcName, testBytecode, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
            it('A() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(["another-proc", testProcName]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createAnyProcedure(testProcName, testBytecode, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");
                }

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),356, "new value should be 356");
            })
            it('A() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([procName+"abc"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createAnyProcedure(testProcName, testBytecode, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
        })
        describe('B() - without data', function () {
            const testProcName = "SysCallTest";
            const testBytecode = Valid.SysCallTest.bytecode;
            const functionSpec = "B()";
            it('B() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap2, cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");
                }
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

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }

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

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
            it('B() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(["another-proc", testProcName]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap2, cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");
                }

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),356, "new value should be 356");
            })
            it('B() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([procName+"abc"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
        })
        describe('C() - with data (function selector)', function () {
            const testProcName = "SysCallTest";
            const testBytecode = Valid.SysCallTest.bytecode;
            const functionSpec = "C()";
            it('C() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap2, cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");
                }

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

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }

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

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),originalValue.toNumber(), `new value should still be ${originalValue.toNumber()}`);
            })
            it('C() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(["another-proc", testProcName]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap2, cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");
                }

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),(originalValue.toNumber() + 1), `new value should be ${originalValue.toNumber()+1}`);
            })
            it('C() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([procName+"abc"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),originalValue.toNumber(), `new value should still be ${originalValue.toNumber()}`);
            })
        })
        describe('D() - with data (function selector and arguments)', function () {
            const testProcName = "Adder";
            const testBytecode = Valid.Adder.bytecode;
            const functionSpec = "D()";
            it('D() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap2, cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");
                }

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),8, `new value should be 8`);
            })
            it('D() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),originalValue.toNumber(), `new value should still be ${originalValue.toNumber()}`);
            })
            it('D() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),originalValue.toNumber(), `new value should still be ${originalValue.toNumber()}`);
            })
            it('D() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(["another-proc", testProcName]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap2, cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");
                }

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),8, `new value should be 8`);
            })
            it('D() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([procName+"abc"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),originalValue.toNumber(), `new value should still be ${originalValue.toNumber()}`);
            })
        })
        describe('E() - with data (function selector and arguments) and return', function () {
            const testProcName = "Adder";
            const testBytecode = Valid.Adder.bytecode;
            const functionSpec = "E()";
            it('E() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap2, cap1]));

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    for (const log of tx3.receipt.logs) {
                        if (log.topics.length > 0) {
                            console.log(`${web3.toAscii(log.topics[0])} - ${log.data} - ${web3.toAscii(log.data)}`);
                        } else {
                            console.log(`${log.topics[0]} - ${log.data} - ${web3.toAscii(log.data)}`);
                        }
                    }

                    assert.equal(valueX.toNumber(),8, `new value should be 8`);
                }
            })
            it('E() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }
            })
            it('E() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }
            })
            it('E() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(["another-proc", testProcName]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap2, cap1]));

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(),8, `new value should be 8`);
                }
            })
            it('E() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([procName+"abc"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }
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
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the first called procedure, which doesn't really do anything
                await kernel.createProcedure("Adder", Valid.Adder.bytecode, beakerlib.Cap.toInput([]));
                // // This is the second called procedure, which requires capabilities
                await kernel.createProcedure("SysCallTest", Valid.SysCallTest.bytecode, beakerlib.Cap.toInput([cap2, cap1]));
                // await kernel.createProcedure("SysCallTestCall", Valid.SysCallTestCall.bytecode, beakerlib.Cap.toInput([cap2, cap1]));

                {
                    await testutils.installEntryProc(kernel);

                    // Procedure keys must occupay the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(),8, `new value should be 8`);
                }
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
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                await kernel.createProcedure("Adder", Valid.Adder.bytecode, beakerlib.Cap.toInput([]));
                await kernel.createProcedure("FirstNestedCall",  Valid.FirstNestedCall.bytecode,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8001,0), new beakerlib.CallCap()]));
                await kernel.createProcedure("SecondNestedCall", Valid.SecondNestedCall.bytecode, beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8002,0), new beakerlib.CallCap()]));
                await kernel.createProcedure("ThirdNestedCall",  Valid.ThirdNestedCall.bytecode,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8003,0), new beakerlib.CallCap()]));
                await kernel.createProcedure("FourthNestedCall", Valid.FourthNestedCall.bytecode, beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8004,0), new beakerlib.CallCap()]));
                await kernel.createProcedure("FifthNestedCall",  Valid.FifthNestedCall.bytecode,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8005,0), new beakerlib.CallCap()]));
                await kernel.createProcedure("SixthNestedCall",  Valid.SixthNestedCall.bytecode,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8006,0), new beakerlib.CallCap()]));

                await kernel.executeProcedure("FirstNestedCall", "G()", "", 32);

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