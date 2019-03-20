const debug = require('debug')
const assert = require('assert')
const fs = require('fs')

const Kernel = artifacts.require('./TestKernel.sol')
const abi = require('ethereumjs-abi')

const beakerlib = require("../../../beakerlib");
const testutils = require("../../testutils.js");

// Valid Contracts
const Valid = {
    Adder: artifacts.require('test/valid/Adder.sol'),
    Multiply: artifacts.require('test/valid/Multiply.sol'),
    Divide: artifacts.require('test/valid/Divide.sol'),
    AccSimple: artifacts.require('test/valid/AccSimple.sol'),
    SysCallTestWrite: artifacts.require('test/valid/SysCallTestWrite.sol'),
    SysCallTestCall: artifacts.require('test/valid/SysCallTestCall.sol'),
    SysCallTestAccCall: artifacts.require('test/valid/SysCallTestAccCall.sol'),
    FirstNestedCall: artifacts.require('test/valid/NestedCalls/FirstNestedCall.sol'),
    SecondNestedCall: artifacts.require('test/valid/NestedCalls/SecondNestedCall.sol'),
    ThirdNestedCall: artifacts.require('test/valid/NestedCalls/ThirdNestedCall.sol'),
    FourthNestedCall: artifacts.require('test/valid/NestedCalls/FourthNestedCall.sol'),
    FifthNestedCall: artifacts.require('test/valid/NestedCalls/FifthNestedCall.sol'),
    SixthNestedCall: artifacts.require('test/valid/NestedCalls/SixthNestedCall.sol'),
    BasicEntryProcedure: artifacts.require('BasicEntryProcedure.sol'),
}

const TestWrite = artifacts.require('test/TestWrite.sol');

// Some notes:
//   * We don't test for non-existant contracts as this is a property of
//     Ethereum itself, not Beaker.
contract('Kernel with entry procedure', function (accounts) {
    describe('Account Call capability', function () {
        const procName = "SysCallTestAccCall";
        const contract = Valid.SysCallTestAccCall;
        const bytecode = Valid.SysCallTestAccCall.bytecode;

        describe('With CallAny = true and SendValue = true', function () {
            const testProcName = "TestWrite";
            const testBytecode = TestWrite.bytecode;
            const testContract = TestWrite;
            const functionSpec = "A(address,uint256)";

            const cap1 = new beakerlib.WriteCap(0x8000,2);
            const cap2 = new beakerlib.LogCap([]);
            const cap3 = new beakerlib.AccCallCap(true, true);
            const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

            it('Should be able to call a contract and retrieve correct testNum', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();
                const amount = 0;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();
                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))
                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer. In
                    // this test we aren't going to actually transfer any funds,
                    // but we want to ensure that no funds are sent regardless
                    // of whether they are available.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64+64);
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 37, "should return correct testNum for AccSimple contract");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance, "balance should be unchanged");
            })
            it('Should be able to call a contract, retrieve correct testNum and transfer 1 wei', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();
                const amount = 1;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();
                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))
                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64+64);
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 37, "should return correct testNum for AccSimple contract");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance+amount, `balance should be increased by ${amount}`);
            })
        })
        describe('With CallAny = true and SendValue = false', function () {
            const testProcName = "TestWrite";
            const testBytecode = TestWrite.bytecode;
            const testContract = TestWrite;
            const functionSpec = "A(address,uint256)";

            const cap1 = new beakerlib.WriteCap(0x8000,2);
            const cap2 = new beakerlib.LogCap([]);
            const cap3 = new beakerlib.AccCallCap(true, false);
            const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

            it('Should be able to call a contract and retrieve correct testNum', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();
                const amount = 0;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();
                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))
                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer. In
                    // this test we aren't going to actually transfer any funds,
                    // but we want to ensure that no funds are sent regardless
                    // of whether they are available.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64+64);
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 37, "should return correct testNum for AccSimple contract");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance, "balance should be unchanged");
            })
            it('Should fail to call a contract, retrieve correct testNum and transfer 1 wei', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();
                const amount = 1;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();
                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))
                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64);
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert(valueX.toNumber() != 37, "should not return the correct testNum");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance, `balance should be remain unchanged`);
            })
        })
        describe('With CallAny = false and SendValue = true', function () {
            const testProcName = "TestWrite";
            const testBytecode = TestWrite.bytecode;
            const testContract = TestWrite;
            const functionSpec = "A(address,uint256)";

            it('Should be able to call a contract and retrieve correct testNum from specified contract', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.AccCallCap(false, true, simple.address);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);


                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))

                const amount = 0;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer. In
                    // this test we aren't going to actually transfer any funds,
                    // but we want to ensure that no funds are sent regardless
                    // of whether they are available.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64+64);
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 37, "should return correct testNum for AccSimple contract");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance, "balance should be unchanged");
            })
            it('Should be able to call a contract, retrieve correct testNum and transfer 1 wei from specified contract', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.AccCallCap(false, true, simple.address);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);


                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))

                const amount = 1;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64+64);
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 37, "should return correct testNum");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance+amount, `balance should be increased by the specified amount`);
            })

            it('Should fail to call a contract and retrieve correct testNum from another contract', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();

                // This is a dummy contract. We will use this address in the
                // capability, but we will try and call the contract we deplyed
                // above. This means we will be calling an address other than
                // the one specified, which should fail.
                const other = await Valid.AccSimple.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.AccCallCap(false, true, other.address);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);


                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))

                // We will also check that the balance of the other contract
                // remains unchanged.
                const otherInitialBalanceRaw = await web3.eth.getBalance(other.address);
                const otherInitialBalance = web3.toDecimal(web3.fromDecimal(otherInitialBalanceRaw))

                const amount = 0;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64);
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert(valueX.toNumber() != 37, "should not return correct testNum");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance, `balance should be remain unchanged`);

                // Check that the balance of the other contract also remains
                // unchanged.
                const otherFinalBalanceRaw = await web3.eth.getBalance(other.address);
                const otherFinalBalance = web3.toDecimal(web3.fromDecimal(otherFinalBalanceRaw))
                assert.equal(otherFinalBalance, otherInitialBalance, `balance of other contract should be remain unchanged`);
            })
            it('Should fail to call a contract, retrieve correct testNum and transfer 1 wei from another contract', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();

                // This is a dummy contract. We will use this address in the
                // capability, but we will try and call the contract we deplyed
                // above. This means we will be calling an address other than
                // the one specified, which should fail.
                const other = await Valid.AccSimple.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.AccCallCap(false, true, other.address);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);


                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))

                // We will also check that the balance of the other contract
                // remains unchanged.
                const otherInitialBalanceRaw = await web3.eth.getBalance(other.address);
                const otherInitialBalance = web3.toDecimal(web3.fromDecimal(otherInitialBalanceRaw))

                const amount = 1;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64);
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert(valueX.toNumber() != 37, "should not return correct testNum");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance, `balance should be remain unchanged`);

                // Check that the balance of the other contract also remains
                // unchanged.
                const otherFinalBalanceRaw = await web3.eth.getBalance(other.address);
                const otherFinalBalance = web3.toDecimal(web3.fromDecimal(otherFinalBalanceRaw))
                assert.equal(otherFinalBalance, otherInitialBalance, `balance of other contract should be remain unchanged`);
            })
        })
        describe('With CallAny = false and SendValue = false', function () {
            const testProcName = "TestWrite";
            const testBytecode = TestWrite.bytecode;
            const testContract = TestWrite;
            const functionSpec = "A(address,uint256)";

            it('Should be able to call a contract and retrieve correct testNum from specified contract', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.AccCallCap(false, false, simple.address);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);


                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))

                const amount = 0;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer. In
                    // this test we aren't going to actually transfer any funds,
                    // but we want to ensure that no funds are sent regardless
                    // of whether they are available.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64+64);
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 37, "should return correct testNum for AccSimple contract");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance, "balance should be unchanged");
            })
            it('Should fail to call a contract, retrieve correct testNum and transfer 1 wei from specified contract', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.AccCallCap(false, false, simple.address);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);


                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))

                const amount = 1;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64);
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert(valueX.toNumber() != 37, "should not return correct testNum");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance, `balance should remain unchanged`);
            })

            it('Should fail to call a contract and retrieve correct testNum from another contract', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();

                // This is a dummy contract. We will use this address in the
                // capability, but we will try and call the contract we deplyed
                // above. This means we will be calling an address other than
                // the one specified, which should fail.
                const other = await Valid.AccSimple.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.AccCallCap(false, false, other.address);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);


                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))

                // We will also check that the balance of the other contract
                // remains unchanged.
                const otherInitialBalanceRaw = await web3.eth.getBalance(other.address);
                const otherInitialBalance = web3.toDecimal(web3.fromDecimal(otherInitialBalanceRaw))

                const amount = 0;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64);
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert(valueX.toNumber() != 37, "should not return correct testNum");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance, `balance should be remain unchanged`);

                // Check that the balance of the other contract also remains
                // unchanged.
                const otherFinalBalanceRaw = await web3.eth.getBalance(other.address);
                const otherFinalBalance = web3.toDecimal(web3.fromDecimal(otherFinalBalanceRaw))
                assert.equal(otherFinalBalance, otherInitialBalance, `balance of other contract should be remain unchanged`);
            })
            it('Should fail to call a contract, retrieve correct testNum and transfer 1 wei from another contract', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();

                // This is a dummy contract. We will use this address in the
                // capability, but we will try and call the contract we deplyed
                // above. This means we will be calling an address other than
                // the one specified, which should fail.
                const other = await Valid.AccSimple.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.AccCallCap(false, false, other.address);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);


                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))

                // We will also check that the balance of the other contract
                // remains unchanged.
                const otherInitialBalanceRaw = await web3.eth.getBalance(other.address);
                const otherInitialBalance = web3.toDecimal(web3.fromDecimal(otherInitialBalanceRaw))

                const amount = 1;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64);
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert(valueX.toNumber() != 37, "should not return correct testNum");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance, `balance should be remain unchanged`);

                // Check that the balance of the other contract also remains
                // unchanged.
                const otherFinalBalanceRaw = await web3.eth.getBalance(other.address);
                const otherFinalBalance = web3.toDecimal(web3.fromDecimal(otherFinalBalanceRaw))
                assert.equal(otherFinalBalance, otherInitialBalance, `balance of other contract should be remain unchanged`);
            })
        })
        describe('With wrong cap', function () {
            const testProcName = "TestWrite";
            const testBytecode = TestWrite.bytecode;
            const testContract = TestWrite;
            const functionSpec = "A(address,uint256)";

            const cap1 = new beakerlib.WriteCap(0x8000,2);
            const cap2 = new beakerlib.LogCap([]);
            const cap3 = new beakerlib.LogCap([]);
            const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

            it('Should fail to call a contract and retrieve correct testNum', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();
                const amount = 0;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();
                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))
                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer. In
                    // this test we aren't going to actually transfer any funds,
                    // but we want to ensure that no funds are sent regardless
                    // of whether they are available.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64);
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert(valueX.toNumber() != 37, "should not return correct testNum for AccSimple contract");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance, "balance should be unchanged");
            })
            it('Should fail to call a contract, retrieve correct testNum and transfer 1 wei', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();
                const amount = 1;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();
                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))
                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64);
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert(valueX.toNumber() != 37, "should not return correct testNum");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance, `balance should be remain unchanged`);
            })
        })
        describe('Without cap', function () {
            const testProcName = "TestWrite";
            const testBytecode = TestWrite.bytecode;
            const testContract = TestWrite;
            const functionSpec = "A(address,uint256)";

            const cap1 = new beakerlib.WriteCap(0x8000,2);
            const cap2 = new beakerlib.LogCap([]);
            const capArray = beakerlib.Cap.toInput([cap1, cap2]);

            it('Should fail to call a contract and retrieve correct testNum', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();
                const amount = 0;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();
                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))
                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer. In
                    // this test we aren't going to actually transfer any funds,
                    // but we want to ensure that no funds are sent regardless
                    // of whether they are available.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64);
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert(valueX.toNumber() != 37, "should not return correct testNum for AccSimple contract");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance, "balance should be unchanged");
            })
            it('Should fail to call a contract, retrieve correct testNum and transfer 1 wei', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();
                const amount = 1;

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                await testutils.installEntryProc(kernel);

                // Test that the AccCall test procedure is correctly installed
                // by querying a value from it.

                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx = await kernel.sendTransaction({data: inputData});

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 7888, "should return the correct test number");
                }

                // Deploy a regular Ethereum contract that is not related to
                // Beaker.
                const simple = await Valid.AccSimple.new();
                const initialBalanceRaw = await web3.eth.getBalance(simple.address);
                const initialBalance = web3.toDecimal(web3.fromDecimal(initialBalanceRaw))
                {
                    // Procedure keys must occupy the first 24 bytes, so must be
                    // padded
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0"))
                        + functionSelectorHash
                        + simple.address.slice(2).padStart(32*2,0) // the address argument for call (32 bytes)
                        + web3.toHex(amount).slice(2).padStart(32*2,0) // the amount argument for call (32 bytes)
                        ;
                    // Here it is important to send some value to the kernel as
                    // well, so that we actually have something to transfer.
                    const tx = await kernel.sendTransaction({data: inputData, value: "0x5"});
                    const valueXRawRaw = await web3.eth.call({to: kernel.address, data: inputData, value: "0x5"});
                    const valueXRaw = "0x"+web3.toHex(valueXRawRaw).slice(2+64);
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert(valueX.toNumber() != 37, "should not return correct testNum");
                }
                const finalBalanceRaw = await web3.eth.getBalance(simple.address);
                const finalBalance = web3.toDecimal(web3.fromDecimal(finalBalanceRaw))
                assert.equal(finalBalance, initialBalance, `balance should be remain unchanged`);
            })
        })
    })
})
