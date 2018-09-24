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
    BasicEntryProcedure: artifacts.require('BasicEntryProcedure.sol'),
}

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

contract('Kernel', function (accounts) {
    describe('Write SysCall Procedure', function () {
        it('S() should succeed when given cap', async function () {
            const kernel = await Kernel.new();

            const capArraySysCallTest = beakerlib.Cap.toInput([
                new beakerlib.WriteCap(0x8500,2),
                new beakerlib.WriteCap(0x8000,0)
            ]);

            const tx1 = await kernel.createProcedure("SysCallTest", Valid.SysCallTest.bytecode, capArraySysCallTest);
            const tx2 = await kernel.createProcedure("Simple", Invalid.Simple.bytecode, []);

            const newValue1 = await kernel.testGetter.call();
            assert.equal(newValue1.toNumber(), 3, "The value should be 3 before the execution");

            const entryProcName = "EntryProcedure";
            const entryProcBytecode = Valid.BasicEntryProcedure.bytecode;
            const capArrayEntryProc = beakerlib.Cap.toInput([
                new beakerlib.WriteCap(0x8000,2),
                new beakerlib.LogCap([]),
                new beakerlib.CallCap()
            ]);

            // Install the entry procedure
            await kernel.createAnyProcedure(entryProcName, entryProcBytecode, capArrayEntryProc);

            // Procedure keys must occupay the first 24 bytes, so must be
            // padded
            const functionSelector = "S()";
            // const functionSelectorHash = web3.sha3(functionSelector);
            const functionSelectorHash = "4be1c796"
            const inputData = web3.fromAscii("SysCallTest".padEnd(24,"\0")) + functionSelectorHash;
            const tx3 = await kernel.sendTransaction({data: inputData});

            // for (const log of tx3.receipt.logs) {
            //     if (log.topics.length > 0) {
            //         console.log(`Log: ${web3.toAscii(log.topics[0])} - ${log.data} - ${web3.toAscii(log.data)}`);
            //     } else {
            //         console.log(`Log: ${log.topics[0]} - ${web3.toAscii(log.data)} - ${log.data}`);
            //     }
            // }

            // The log value is 32 bytes log so we pad it out with nulls
            const expectedLogValue = "BasicEntryProcedureFallback".padEnd(32,'\0');
            // Should be trimEnd, but I left it as trim in case you don't
            // have node 10
            assert.equal(web3.toAscii(tx3.receipt.logs[0].data),expectedLogValue, "Should receive a log from entry procedure");

            const newValue4 = await kernel.testGetter.call();
            assert.equal(newValue4.toNumber(), 4, "The value should be 4 after the execution");

        })
        it('S() should fail when not given cap', async function () {
            const kernel = await Kernel.new();

            const tx1 = await kernel.createProcedure("SysCallTest", Valid.SysCallTest.bytecode, []);
            const tx2 = await kernel.createProcedure("Simple", Invalid.Simple.bytecode, []);

            const newValue1 = await kernel.testGetter.call();
            assert.equal(newValue1.toNumber(), 3, "The value should be 3 before the execution");

            const entryProcName = "EntryProcedure";
            const entryProcBytecode = Valid.BasicEntryProcedure.bytecode;
            const capArrayEntryProc = beakerlib.Cap.toInput([
                new beakerlib.WriteCap(0x8000,2),
                new beakerlib.LogCap([]),
                new beakerlib.CallCap()
            ]);

            // Install the entry procedure
            await kernel.createAnyProcedure(entryProcName, entryProcBytecode, capArrayEntryProc);

            // Procedure keys must occupay the first 24 bytes, so must be
            // padded
            const functionSelector = "S()";
            // const functionSelectorHash = web3.sha3(functionSelector);
            const functionSelectorHash = "4be1c796"
            const inputData = web3.fromAscii("SysCallTest".padEnd(24,"\0")) + functionSelectorHash;
            const tx3 = await kernel.sendTransaction({data: inputData});

            // for (const log of tx3.receipt.logs) {
            //     if (log.topics.length > 0) {
            //         console.log(`Log: ${web3.toAscii(log.topics[0])} - ${log.data} - ${web3.toAscii(log.data)}`);
            //     } else {
            //         console.log(`Log: ${log.topics[0]} - ${web3.toAscii(log.data)} - ${log.data}`);
            //     }
            // }

            // The log value is 32 bytes log so we pad it out with nulls
            const expectedLogValue = "BasicEntryProcedureFallback".padEnd(32,'\0');
            // Should be trimEnd, but I left it as trim in case you don't
            // have node 10

            const newValue4 = await kernel.testGetter.call();
            assert.equal(newValue4.toNumber(), 3, "The value should still be 3 after the execution");
        })
        it('S() should fail when trying to write to an address below its cap', async function () {
            const kernel = await Kernel.new();

            const capArraySysCallTest = beakerlib.Cap.toInput([
                new beakerlib.WriteCap(0x8500,2),
                new beakerlib.WriteCap(0x8001,0)
            ]);

            const tx1 = await kernel.createProcedure("SysCallTest", Valid.SysCallTest.bytecode, capArraySysCallTest);
            const tx2 = await kernel.createProcedure("Simple", Invalid.Simple.bytecode, []);

            const newValue1 = await kernel.testGetter.call();
            assert.equal(newValue1.toNumber(), 3, "The value should be 3 before the execution");

            const entryProcName = "EntryProcedure";
            const entryProcBytecode = Valid.BasicEntryProcedure.bytecode;
            const capArrayEntryProc = beakerlib.Cap.toInput([
                new beakerlib.WriteCap(0x8000,2),
                new beakerlib.LogCap([]),
                new beakerlib.CallCap()
            ]);

            // Install the entry procedure
            await kernel.createAnyProcedure(entryProcName, entryProcBytecode, capArrayEntryProc);

            // Procedure keys must occupay the first 24 bytes, so must be
            // padded
            const functionSelector = "S()";
            // const functionSelectorHash = web3.sha3(functionSelector);
            const functionSelectorHash = "4be1c796"
            const inputData = web3.fromAscii("SysCallTest".padEnd(24,"\0")) + functionSelectorHash;
            const tx3 = await kernel.sendTransaction({data: inputData});

            // for (const log of tx3.receipt.logs) {
            //     if (log.topics.length > 0) {
            //         console.log(`Log: ${web3.toAscii(log.topics[0])} - ${log.data} - ${web3.toAscii(log.data)}`);
            //     } else {
            //         console.log(`Log: ${log.topics[0]} - ${web3.toAscii(log.data)} - ${log.data}`);
            //     }
            // }

            // The log value is 32 bytes log so we pad it out with nulls
            const expectedLogValue = "BasicEntryProcedureFallback".padEnd(32,'\0');
            // Should be trimEnd, but I left it as trim in case you don't
            // have node 10

            const newValue4 = await kernel.testGetter.call();
            assert.equal(newValue4.toNumber(), 3, "The value should still be 3 after the execution");
        })
    })
})