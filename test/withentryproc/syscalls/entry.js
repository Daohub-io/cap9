const debug = require('debug')
const assert = require('assert')

const Kernel = artifacts.require('./TestKernel.sol')
const abi = require('ethereumjs-abi')

const beakerlib = require("../../../beakerlib");
const testutils = require("../../testutils.js");

const SysCallTestEntry = artifacts.require('test/SysCallTestEntry.sol')
const TestWrite = artifacts.require('test/TestWrite.sol');

contract('Kernel with entry procedure', function (accounts) {
    describe('Entry capability', function () {
        const procName = "SysCallTestEntry";
        const contract = SysCallTestEntry;
        const bytecode = SysCallTestEntry.bytecode;

        describe('A() - call procedure with entry cap', function () {
            const testProcName = "TestWrite";
            const testBytecode = TestWrite.bytecode;
            const testContract = TestWrite;

            it('A() should succeed when given cap and correct procedure Id', async function () {
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.SetEntryCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);

                // This is the procedure that will be entry procedure first
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the procedure that will be set as entry
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                // Set the "SysCallTestEntry" as entry
                await kernel.setEntryProcedure(procName);

                // Call Old Entry Procedure
                {
                    // Procedure keys must occupay the first 24 bytes, so must be padded
                    const functionSelectorHash = web3.sha3("A()").slice(2,10);
                    const inputData = "0x" + functionSelectorHash;
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);

                    assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");
                }

                // Check that entry procedure has changed
                let entry_raw = await kernel.getEntryProcedure.call();
                let new_entry_proc = web3.toAscii(web3.toHex(entry_raw)).replace(/\0.*$/, '')
                assert.equal(new_entry_proc, "TestWrite")
            })

            it('A() should fail when given cap but invalid procedure Id', async function () {
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.SetEntryCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                const deployedContract = await testutils.deployedTrimmed(contract);

                // This is the procedure that will be entry procedure first
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                // Set the "SysCallTestEntry" as entry
                await kernel.setEntryProcedure(procName);

                // Call Old Entry Procedure
                {
                    // Procedure keys must occupay the first 24 bytes, so must be padded
                    const functionSelectorHash = web3.sha3("A()").slice(2,10);
                    const inputData = "0x" + functionSelectorHash;

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const tx3 = await kernel.sendTransaction({data: inputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 1, "should fail with errcode");
                }

                // Check that entry procedure has changed
                let entry_raw = await kernel.getEntryProcedure.call();
                let new_entry_proc = web3.toAscii(web3.toHex(entry_raw)).replace(/\0.*$/, '')
                assert.equal(new_entry_proc, procName)
            })
        })
    })
})
