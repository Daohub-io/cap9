const debug = require('debug')
const assert = require('assert')

const Kernel = artifacts.require('./TestKernel.sol')
const abi = require('ethereumjs-abi')

const beakerlib = require("../../../beakerlib");
const testutils = require("../../testutils.js");

// Valid Contracts
const Valid = {
    Adder: artifacts.require('test/valid/Adder.sol'),
    Simple: artifacts.require('test/valid/Simple.sol'),
    Multiply: artifacts.require('test/valid/Multiply.sol'),
    Divide: artifacts.require('test/valid/Divide.sol'),
    SysCallTestWrite: artifacts.require('test/valid/SysCallTestWrite.sol'),
    SysCallTestCall: artifacts.require('test/valid/SysCallTestCall.sol'),
    SysCallTestProcRegister: artifacts.require('test/valid/SysCallTestProcRegister.sol'),
    SysCallTestSetEntry: artifacts.require('test/valid/SysCallTestSetEntry.sol'),
    BasicEntryProcedure: artifacts.require('BasicEntryProcedure.sol'),
}

const TestWrite = artifacts.require('test/TestWrite.sol');

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

contract('Kernel with entry procedure', function (accounts) {
    describe('Set the entry procedure', function () {
        const entryProcName = "EntryProcedure";
        describe('When sufficient caps given', function () {
            const contractA = Valid.SysCallTestSetEntry;
            // We use Valid.Simple for Procedure B as we never need to actually
            // execute it.
            const contractB = Valid.Simple;

            it('Set the entry procedure to Procedure B', async function () {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Save the initial state of the procedure table
                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A as the entry procedure
                const procAName = "ProcedureA";
                const caps = [
                    new beakerlib.SetEntryCap(),
                    new beakerlib.WriteCap(0x8000,2),
                    new beakerlib.WriteCap(0x8990,5)
                ];
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                {
                    await kernel.registerAnyProcedure(procAName, deployedEntryProc.address, []);
                    for (const cap of caps) {
                        await kernel.addCap(procAName, beakerlib.Cap.toInput([cap]))
                    }
                    // const tx = await kernel.addCap(procAName, capArrayEntryProcAdd)
                    // for (const log of tx.receipt.logs) {
                    //     // console.log(`${log.topics} - ${log.data}`);
                    //     console.log(`${log.topics} - ${log.data}`);
                    //     try {
                    //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                    //     } catch(e) {
                    //         // console.log(`${log.topics} - ${log.data}`);
                    //         console.log("non-ascii");
                    //     }
                    // }
                }

                // {
                //     const procTableData = await kernel.returnProcedureTable.call();
                //     const procTable = beakerlib.ProcedureTable.parse(procTableData);
                //     // console.log("Kernel Address:", kernel.address)
                //     console.log(beakerlib.ProcedureTable.stringify(procTable));
                // }

                // Get the current state of the procedure list and check that
                // Procedure A has been successfully added.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procAName), "Procedure A should be included in the procedure list")
                    assert.equal(procedures.length, 2, "There should be exactly 2 procedures");
                }

                const procBName = "ProcedureB";
                const deployedContractB = await testutils.deployedTrimmed(contractB);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procBName, deployedContractB.address, []);

                // Test that Procedure A executes correctly by calling the
                // testNum() function of that contract.
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 773, "should receive the correct test number");
                }

                // Check that initially the entry procedure is "EntryProcedure".
                {
                    const entryProc = await kernel.getEntryProcedure();
                    const entryProcName = web3.toAscii(entryProc).split('\0')[0];
                    assert.equal(entryProcName, "EntryProcedure", "Initially the entry procedure should be EntryProcedure");
                }
                // Call the SetEntry function to set the entry procedure to
                // Procedure B.
                {
                    const functionSelectorHash = web3.sha3("SetEntry(bytes24)").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;

                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure to set as the entry (32 bytes)

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    const tx3 = await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Check that the entry procedure has been changed to
                // "ProcedureB."
                {
                    const entryProc = await kernel.getEntryProcedure();
                    const entryProcName = web3.toAscii(entryProc).split('\0')[0];
                    assert.equal(entryProcName, "ProcedureB", "Initially the entry procedure should be ProcedureB");
                }
            })
            it('Set the entry procedure to a non-existent', async function () {
                const nonExistentName = "NonExistent";
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Save the initial state of the procedure table
                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A as the entry procedure
                const procAName = "ProcedureA";
                const caps = [
                    new beakerlib.SetEntryCap()
                ];
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                await kernel.registerAnyProcedure(procAName, deployedEntryProc.address, []);
                for (const cap of caps) {
                    await kernel.addCap(procAName, beakerlib.Cap.toInput([cap]))
                }
                // Get the current state of the procedure list and check that
                // Procedure A has been successfully added.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procAName), "Procedure A should be included in the procedure list")
                    assert.equal(procedures.length, 2, "There should be exactly 2 procedures");
                }

                const procBName = "ProcedureB";
                const deployedContractB = await testutils.deployedTrimmed(contractB);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procBName, deployedContractB.address, []);

                // Test that Procedure A executes correctly by calling the
                // testNum() function of that contract.
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 773, "should receive the correct test number");
                }

                // Check that initially the entry procedure is "EntryProcedure".
                {
                    const entryProc = await kernel.getEntryProcedure();
                    const entryProcName = web3.toAscii(entryProc).split('\0')[0];
                    assert.equal(entryProcName, "EntryProcedure", "Initially the entry procedure should be EntryProcedure");
                }
                // Call the SetEntry function to set the entry procedure to
                // Procedure B.
                {
                    const functionSelectorHash = web3.sha3("SetEntry(bytes24)").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;

                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(nonExistentName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure to set as the entry (32 bytes)

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    const tx3 = await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Check that the entry procedure has been changed to
                // "ProcedureB."
                {
                    const entryProc = await kernel.getEntryProcedure();
                    const entryProcName = web3.toAscii(entryProc).split('\0')[0];
                    assert.equal(entryProcName, nonExistentName, `The entry procedure should be ${nonExistentName}`);
                }
            })
        })
        describe('When insufficient caps given', function () {
            const contractA = Valid.SysCallTestSetEntry;
            // We use Valid.Simple for Procedure B as we never need to actually
            // execute it.
            const contractB = Valid.Simple;

            it('Fail to set the entry procedure to Procedure B', async function () {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Save the initial state of the procedure table
                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A as the entry procedure
                const procAName = "ProcedureA";
                const caps = [
                    new beakerlib.DeleteCap()
                ];
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                await kernel.registerAnyProcedure(procAName, deployedEntryProc.address, []);
                for (const cap of caps) {
                    await kernel.addCap(procAName, beakerlib.Cap.toInput([cap]))
                }
                // Get the current state of the procedure list and check that
                // Procedure A has been successfully added.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procAName), "Procedure A should be included in the procedure list")
                    assert.equal(procedures.length, 2, "There should be exactly 2 procedures");
                }

                const procBName = "ProcedureB";
                const deployedContractB = await testutils.deployedTrimmed(contractB);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procBName, deployedContractB.address, []);

                // Test that Procedure A executes correctly by calling the
                // testNum() function of that contract.
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 773, "should receive the correct test number");
                }

                // Check that initially the entry procedure is "EntryProcedure".
                {
                    const entryProc = await kernel.getEntryProcedure();
                    const entryProcName = web3.toAscii(entryProc).split('\0')[0];
                    assert.equal(entryProcName, "EntryProcedure", "Initially the entry procedure should be EntryProcedure");
                }
                // Call the SetEntry function to set the entry procedure to
                // Procedure B.
                {
                    const functionSelectorHash = web3.sha3("SetEntry(bytes24)").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;

                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure to set as the entry (32 bytes)

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    const tx3 = await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert(valueX.toNumber() !== 0, "Should return a non-zero error code");
                }

                // Check that the entry procedure has been changed to
                // "ProcedureB."
                {
                    const entryProc = await kernel.getEntryProcedure();
                    const entryProcName = web3.toAscii(entryProc).split('\0')[0];
                    assert.equal(entryProcName, "EntryProcedure", "Initially the entry procedure should still be EntryProcedure");
                }
            })
        })
    })
})
