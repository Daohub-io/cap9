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
    SysCallTestProcDelete: artifacts.require('test/valid/SysCallTestProcDelete.sol'),
    BasicEntryProcedure: artifacts.require('BasicEntryProcedure.sol'),
}

const TestWrite = artifacts.require('test/TestWrite.sol');

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

contract('Kernel with entry procedure', function (accounts) {
    describe('Delete capability', function () {
        const entryProcName = "EntryProcedure";

        describe('When given cap to delete a specific procedure Id', function () {
            // * Introduces Procedure A and Procedure B into the procedure table.
            // * Procedure A is designated a procedure delete capability (type `0x3`)
            //   that allows it to delete any procedure iff it has id `X`.
            // * Procedure B is not given any capabilities.
            // * Procedure A removes Procedure B from the procedure table by
            //   invoking it's capability **iff** Procedure B has a procedure id `X`.

            // Procedure A is a contract with code that deletes a requested
            // procedure.
            const contractA = Valid.SysCallTestProcDelete;
            // We use Valid.Simple for Procedure B as we never need to actually
            // execute it.
            const contractB = Valid.Simple;

            it('Delete procedure with matching id', async function() {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Save the initial state of the procedure table
                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A as the entry procedure
                const procAName = "ProcedureA";
                const procBName = "ProcedureB";
                const deployedContractB = await testutils.deployedTrimmed(contractB);
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);

                // Give ProcedureA a delete capability to remove only ProcedureB
                const capArrayEntryProc = beakerlib.Cap.toInput([
                    new beakerlib.DeleteCap([procBName])
                ]);

                // This uses a direct call to the kernel
                await kernel.registerAnyProcedure(procAName, deployedEntryProc.address, capArrayEntryProc);
                await kernel.registerAnyProcedure(procBName, deployedContractB.address, []);

                // Call the delete function to delete Procedure B
                {
                    const functionSelectorHash = web3.sha3("Delete(bytes24)").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;

                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name argument for register (32 bytes)
                    // console.log(manualInputData)
                    // when using web3 1.0 this will be good
                    // try {
                    //     console.log(deployedContract.methods.B(testProcName,deployedTestContract.address,[]).data)
                    // } catch (e) {
                    //     console.log(e)
                    // }
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    const tx3 = await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }


                // Get the current state of the procedure list and check that
                // Procedure B has been successfully deleted.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(!procedures.includes(procBName), "Procedure B should no longer be included in the procedure list")
                    assert(procedures.includes(procAName), "Procedure A should still be included in the procedure list")
                    assert(procedures.includes(entryProcName), "Entry procedure should still be included in the procedure list")
                    assert.equal(procedures.length, 2, "There should be exactly 2 procedures");
                }

            })
            it('Fails to delete procedure if non-matching id', async function() {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Save the initial state of the procedure table
                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A as the entry procedure
                const procAName = "ProcedureA";
                const procBName = "ProcedureB";
                const procCName = "BOOO";

                const deployedContractC = await testutils.deployedTrimmed(contractA);
                const deployedContractB = await testutils.deployedTrimmed(contractB);
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);

                // Give ProcedureA a delete capability to remove a procedure Id different from ProcedureB
                const capArrayEntryProc = beakerlib.Cap.toInput([
                    new beakerlib.DeleteCap([procCName])
                ]);

                // This uses a direct call to the kernel
                await kernel.registerAnyProcedure(procAName, deployedEntryProc.address, capArrayEntryProc);
                await kernel.registerAnyProcedure(procBName, deployedContractB.address, []);
                await kernel.registerAnyProcedure(procCName, deployedContractB.address, []);

                // Call the delete function to delete Procedure B, not Procedure C
                {
                    const functionSelectorHash = web3.sha3("Delete(bytes24)").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;

                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name argument for register (32 bytes)
                    // console.log(manualInputData)
                    // when using web3 1.0 this will be good
                    // try {
                    //     console.log(deployedContract.methods.B(testProcName,deployedTestContract.address,[]).data)
                    // } catch (e) {
                    //     console.log(e)
                    // }
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    const tx3 = await kernel.sendTransaction({data: manualInputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    // assert.notEqual(valueX.toNumber(), 0, "Should return a non-zero error code");
                }

                // Get the current state of the procedure list and check that
                // Both Procedure B and C should not have been deleted.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procCName), "Procedure C should still be included in the procedure list")
                    assert(procedures.includes(procBName), "Procedure B should still be included in the procedure list")
                    assert(procedures.includes(procAName), "Procedure A should still be included in the procedure list")
                    assert(procedures.includes(entryProcName), "Entry procedure should still be included in the procedure list")
                    assert.equal(procedures.length, 4, "There should still be exactly 4 procedures");
                }
            })
        })
        describe('When given cap to delete all (*)', function () {
            // * Introduces Procedure A and Procedure B into the procedure
            //   table.
            // * Procedure A is designated a procedure delete capability (type
            //   `0x2`) that allows it to delete Procedure B based on it's id.
            // * Procedure B is not given any capabilities.
            // * Procedure A removes Procedure B from the procedure table by
            //   invoking it's capability.

            // Procedure A is a contract with code that deletes a requested
            // procedure.
            const contractA = Valid.SysCallTestProcDelete;
            // We use Valid.Simple for Procedure B as we never need to actually
            // execute it.
            const contractB = Valid.Simple;

            it('Delete a procedure', async function () {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Save the initial state of the procedure table
                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A as the entry procedure
                const procAName = "ProcedureA";
                const capArrayEntryProc = beakerlib.Cap.toInput([
                    new beakerlib.DeleteCap()
                ]);
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                await kernel.registerAnyProcedure(procAName, deployedEntryProc.address, capArrayEntryProc);

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

                // Get the current state of the procedure list and check that
                // Procedure B has been successfully added.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procBName), "Procedure B should be included in the procedure list")
                    assert.equal(procedures.length, 3, "There should be exactly 3 procedures");
                }
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
                    assert.equal(valueX.toNumber(), 982, "should receive the correct test number");
                }

                // Call the delete function to delete Procedure B
                {
                    const functionSelectorHash = web3.sha3("Delete(bytes24)").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;

                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name argument for register (32 bytes)
                    // console.log(manualInputData)
                    // when using web3 1.0 this will be good
                    // try {
                    //     console.log(deployedContract.methods.B(testProcName,deployedTestContract.address,[]).data)
                    // } catch (e) {
                    //     console.log(e)
                    // }


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    const tx3 = await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Get the current state of the procedure list and check that
                // Procedure B has been successfully deleted.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(!procedures.includes(procBName), "Procedure B should no longer be included in the procedure list")
                    assert(procedures.includes(procAName), "Procedure A should still be included in the procedure list")
                    assert(procedures.includes(entryProcName), "Entry procedure should still be included in the procedure list")
                    assert.equal(procedures.length, 2, "There should be exactly 2 procedures");
                }
            })
            it('Delete itself', async function () {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Save the initial state of the procedure table
                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A as the entry procedure
                const procAName = "ProcedureA";
                const capArrayEntryProc = beakerlib.Cap.toInput([
                    new beakerlib.DeleteCap()
                ]);
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                await kernel.registerAnyProcedure(procAName, deployedEntryProc.address, capArrayEntryProc);

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

                // Get the current state of the procedure list and check that
                // Procedure B has been successfully added.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procBName), "Procedure B should be included in the procedure list")
                    assert.equal(procedures.length, 3, "There should be exactly 3 procedures");
                }
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
                    assert.equal(valueX.toNumber(), 982, "should receive the correct test number");
                }

                // Call the delete function to delete Procedure B
                {
                    const functionSelectorHash = web3.sha3("Delete(bytes24)").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;

                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procAName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name argument for register (32 bytes)
                    // console.log(manualInputData)
                    // when using web3 1.0 this will be good
                    // try {
                    //     console.log(deployedContract.methods.B(testProcName,deployedTestContract.address,[]).data)
                    // } catch (e) {
                    //     console.log(e)
                    // }


                    const tx3 = await kernel.sendTransaction({data: manualInputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    // const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueXRaw, "0x", "Should return nothing");
                }


                // Get the current state of the procedure list and check that
                // Procedure B has been successfully deleted.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(!procedures.includes(procAName), "Procedure A should no longer be included in the procedure list")
                    assert(procedures.includes(procBName), "Procedure B should still be included in the procedure list")
                    assert(procedures.includes(entryProcName), "Entry procedure should still be included in the procedure list")
                    assert.equal(procedures.length, 2, "There should be exactly 2 procedures");
                }
            })
            it('Fail to delete the entry procedure', async function () {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Save the initial state of the procedure table
                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A as the entry procedure
                const procAName = "ProcedureA";
                const capArrayEntryProc = beakerlib.Cap.toInput([
                    new beakerlib.DeleteCap()
                ]);
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                await kernel.registerAnyProcedure(procAName, deployedEntryProc.address, capArrayEntryProc);

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

                // Get the current state of the procedure list and check that
                // Procedure B has been successfully added.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procBName), "Procedure B should be included in the procedure list")
                    assert.equal(procedures.length, 3, "There should be exactly 3 procedures");
                }
                // Test that Procedure A executes correctly by calling the
                // testNum() function of that contract.
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 982, "should receive the correct test number");
                }

                // Call the delete function to delete the entry procedure
                {
                    const functionSelectorHash = web3.sha3("Delete(bytes24)").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;

                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(entryProcName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name argument for delete (32 bytes)
                    // console.log(manualInputData)
                    // when using web3 1.0 this will be good
                    // try {
                    //     console.log(deployedContract.methods.B(testProcName,deployedTestContract.address,[]).data)
                    // } catch (e) {
                    //     console.log(e)
                    // }

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    const tx = await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.notEqual(valueX.toNumber(), 0, "Should not return a zero error code");
                }

                // Get the current state of the procedure list and check that
                // Procedure B has been successfully deleted.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procBName), "Procedure B should still be included in the procedure list")
                    assert(procedures.includes(procAName), "Procedure A should still be included in the procedure list")
                    assert(procedures.includes(entryProcName), "Entry procedure should still be included in the procedure list")
                    assert.equal(procedures.length, 3, "There should still be exactly 3 procedures");
                }
            })
            it('Fail to delete non-existent procedure', async function () {
                const nonExistentName = "NonExistant";
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Save the initial state of the procedure table
                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A as the entry procedure
                const procAName = "ProcedureA";
                const capArrayEntryProc = beakerlib.Cap.toInput([
                    new beakerlib.DeleteCap()
                ]);
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                await kernel.registerAnyProcedure(procAName, deployedEntryProc.address, capArrayEntryProc);

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

                // Get the current state of the procedure list and check that
                // Procedure B has been successfully added.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procBName), "Procedure B should be included in the procedure list")
                    assert.equal(procedures.length, 3, "There should be exactly 3 procedures");
                    assert(!procedures.includes(nonExistentName), "Non-Existent procedure should not be included in the procedure list")
                }
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
                    assert.equal(valueX.toNumber(), 982, "should receive the correct test number");
                }

                // Call the delete function to delete Procedure B
                {
                    const functionSelectorHash = web3.sha3("Delete(bytes24)").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;

                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(nonExistentName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name argument for register (32 bytes)
                    // console.log(manualInputData)
                    // when using web3 1.0 this will be good
                    // try {
                    //     console.log(deployedContract.methods.B(testProcName,deployedTestContract.address,[]).data)
                    // } catch (e) {
                    //     console.log(e)
                    // }


                    const tx3 = await kernel.sendTransaction({data: manualInputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert(valueX.toNumber() != 0, "Should return a non-zero error code");
                }


                // Get the current state of the procedure list and check that no
                // procedures have been deleted.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procBName), "Procedure B should still be included in the procedure list")
                    assert(procedures.includes(procAName), "Procedure A should still be included in the procedure list")
                    assert(procedures.includes(entryProcName), "Entry procedure should still be included in the procedure list")
                    assert(!procedures.includes(nonExistentName), "Non-Existent procedure should still not be included in the procedure list")
                    assert.equal(procedures.length, 3, "There should be exactly 3 procedures");
                }
            })
        })
        describe('When insufficient caps given', function () {
            // * Introduces Procedure A and Procedure B into the procedure
            //   table.
            // * Procedure A is designated a procedure delete capability (type
            //   `0x2`) that allows it to delete Procedure B based on it's id.
            // * Procedure B is not given any capabilities.
            // * Procedure A removes Procedure B from the procedure table by
            //   invoking it's capability.

            // Procedure A is a contract with code that deletes a requested
            // procedure.
            const contractA = Valid.SysCallTestProcDelete;
            // We use Valid.Simple for Procedure B as we never need to actually
            // execute it.
            const contractB = Valid.Simple;

            it('Fail to delete a procedure', async function () {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Save the initial state of the procedure table
                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A as the entry procedure
                const procAName = "ProcedureA";
                const capArrayEntryProc = beakerlib.Cap.toInput([
                    // Give the procedure a call cap, not a delete cap
                    new beakerlib.CallCap()
                ]);
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                await kernel.registerAnyProcedure(procAName, deployedEntryProc.address, capArrayEntryProc);

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

                // Get the current state of the procedure list and check that
                // Procedure B has been successfully added.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procBName), "Procedure B should be included in the procedure list")
                    assert.equal(procedures.length, 3, "There should be exactly 3 procedures");
                }
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
                    assert.equal(valueX.toNumber(), 982, "should receive the correct test number");
                }

                // Call the delete function to delete Procedure B
                {
                    const functionSelectorHash = web3.sha3("Delete(bytes24)").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;

                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name argument for register (32 bytes)
                    // console.log(manualInputData)
                    // when using web3 1.0 this will be good
                    // try {
                    //     console.log(deployedContract.methods.B(testProcName,deployedTestContract.address,[]).data)
                    // } catch (e) {
                    //     console.log(e)
                    // }


                    const tx3 = await kernel.sendTransaction({data: manualInputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert(valueX.toNumber() !== 0, "Should return a non-zero error code");
                }


                // Get the current state of the procedure list and check that
                // Procedure B has been successfully deleted.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procBName), "Procedure B should still be included in the procedure list")
                    assert(procedures.includes(procAName), "Procedure A should still be included in the procedure list")
                    assert.equal(procedures.length, 3, "There should be exactly 3 procedures");
                }
            })
            it('Fail to delete itself', async function () {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Save the initial state of the procedure table
                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A as the entry procedure
                const procAName = "ProcedureA";
                const capArrayEntryProc = beakerlib.Cap.toInput([
                    // Give the procedure a call cap, not a delete cap
                    new beakerlib.CallCap()
                ]);
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                await kernel.registerAnyProcedure(procAName, deployedEntryProc.address, capArrayEntryProc);

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

                // Get the current state of the procedure list and check that
                // Procedure B has been successfully added.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procBName), "Procedure B should be included in the procedure list")
                    assert.equal(procedures.length, 3, "There should be exactly 3 procedures");
                }
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
                    assert.equal(valueX.toNumber(), 982, "should receive the correct test number");
                }

                // Call the delete function to delete Procedure B
                {
                    const functionSelectorHash = web3.sha3("Delete(bytes24)").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;

                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procAName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name argument for register (32 bytes)
                    // console.log(manualInputData)
                    // when using web3 1.0 this will be good
                    // try {
                    //     console.log(deployedContract.methods.B(testProcName,deployedTestContract.address,[]).data)
                    // } catch (e) {
                    //     console.log(e)
                    // }


                    const tx3 = await kernel.sendTransaction({data: manualInputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert(valueX.toNumber() !== 0, "Should return a non-zero error code");
                }


                // Get the current state of the procedure list and check that
                // Procedure B has been successfully deleted.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procAName), "Procedure A should still be included in the procedure list")
                    assert(procedures.includes(procBName), "Procedure B should still be included in the procedure list")
                    assert.equal(procedures.length, 3, "There should be exactly 3 procedures");
                }
            })
            it('Fail to delete the entry procedure', async function () {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Save the initial state of the procedure table
                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A as the entry procedure
                const procAName = "ProcedureA";
                const capArrayEntryProc = beakerlib.Cap.toInput([
                    new beakerlib.DeleteCap()
                ]);
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                await kernel.registerAnyProcedure(procAName, deployedEntryProc.address, capArrayEntryProc);

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

                // Get the current state of the procedure list and check that
                // Procedure B has been successfully added.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procBName), "Procedure B should be included in the procedure list")
                    assert.equal(procedures.length, 3, "There should be exactly 3 procedures");
                }
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
                    assert.equal(valueX.toNumber(), 982, "should receive the correct test number");
                }

                // Call the delete function to delete Procedure B
                {
                    const functionSelectorHash = web3.sha3("Delete(bytes24)").slice(2,10);
                    const inputData = web3.fromAscii(procAName.padEnd(24,"\0")) + functionSelectorHash;

                    const manualInputData = web3.fromAscii(procBName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(entryProcName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name argument for register (32 bytes)
                    // console.log(manualInputData)
                    // when using web3 1.0 this will be good
                    // try {
                    //     console.log(deployedContract.methods.B(testProcName,deployedTestContract.address,[]).data)
                    // } catch (e) {
                    //     console.log(e)
                    // }


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    const tx3 = await kernel.sendTransaction({data: manualInputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert(valueX.toNumber() !== 0, "Should return a non-zero error code");
                }


                // Get the current state of the procedure list and check that
                // Procedure B has been successfully deleted.
                {
                    const proceduresRaw = await kernel.listProcedures.call();
                    const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                    assert(procedures.includes(procBName), "Procedure B should still be included in the procedure list")
                    assert(procedures.includes(procAName), "Procedure A should still be included in the procedure list")
                    assert(procedures.includes(entryProcName), "Entry procedure should still be included in the procedure list")
                    assert.equal(procedures.length, 3, "There should be exactly 3 procedures");
                }
            })
        })
    })
})
