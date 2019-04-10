const debug = require('debug')
const assert = require('assert')

const Kernel = artifacts.require('./TestKernel.sol')
const abi = require('ethereumjs-abi')

const beakerlib = require("../../beakerlib");
const testutils = require("../testutils.js");

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
    BasicEntryProcedure: artifacts.require('BasicEntryProcedure.sol'),
}

const TestWrite = artifacts.require('test/TestWrite.sol');

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

contract('Kernel with entry procedure', function (accounts) {
    describe('Install entry procedure', function () {
        const entryProcName = "EntryProcedure";
        const entryProcBytecode = Valid.BasicEntryProcedure.bytecode;

        describe('A()', function () {
            it('A() should succeed when given cap', async function () {


                // Install the entry procedure
                const kernel = await testutils.deployTestKernel();
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
