const debug = require('debug')
const assert = require('assert')

const Kernel = artifacts.require('./Kernel.sol')
const abi = require('ethereumjs-abi')

const beakerlib = require("../beakerlib");

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
    describe('Install entry procedure', function () {
        const entryProcName = "EntryProcedure";
        const entryProcBytecode = Valid.BasicEntryProcedure.bytecode;

        describe('A()', function () {
            it('A() should succeed when given cap', async function () {
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // Install the entry procedure
                const tx1 = await kernel.createAnyProcedure(entryProcName, entryProcBytecode, capArray);
                // We want to send an empty transaction. With an empty
                // transaction, the following things should occur:
                //   1. The payload should get forwared to the entry
                //     procedure, which is "contracts/BasicEntryProcedure.sol".
                //   2. The fallback function of the entry procedure will be
                //     called, which simply logs "BasicEntryProcedureFallback"
                //     with no topics.
                const tx2 = await kernel.sendTransaction();

                // console.log(tx2)
                // for (const log of tx2.receipt.logs) {
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
                assert.equal(web3.toAscii(tx2.receipt.logs[0].data),expectedLogValue, "Should receive a log from entry procedure");
            })
        })
    })
})