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
    SysCallTestCaps: artifacts.require('test/valid/SysCallTestCaps.sol'),
    SysCallTestSetEntry: artifacts.require('test/valid/SysCallTestSetEntry.sol'),
    BasicEntryProcedure: artifacts.require('BasicEntryProcedure.sol'),
}

const TestWrite = artifacts.require('test/TestWrite.sol');

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

contract('Kernel with entry procedure', function (accounts) {
    describe('Add caps to Procedure B using Procedure A', function () {
        describe('When sufficient caps given', function () {
            // Procedure A is contract which can add caps to another contract
            const contractA = Valid.SysCallTestCaps;
            // We use a contract which can test write calls, this is so we can
            // check that caps can be used properly.
            const contractB = Valid.SysCallTestWrite;

            it('Give write cap to Procedure B', async function () {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A
                const procAName = "ProcedureA";
                const caps = [
                    new beakerlib.PushCap()
                ];
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                {
                    await kernel.registerAnyProcedure(procAName, deployedEntryProc.address);
                    for (const cap of caps) {
                        await kernel.addCap(procAName, beakerlib.Cap.toInput([cap]))
                    }
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
                    assert.equal(valueX.toNumber(), 567, "should receive the correct test number");
                }

                // Install Procedure B
                const procBName = "ProcedureB";
                const deployedContractB = await testutils.deployedTrimmed(contractB);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procBName, deployedContractB.address);

                // Test that Procedure B executes correctly by calling the
                // testNum() function of that contract.
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procBName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 986, "should receive the correct test number");
                }

                // Check that initially Procedure B has no capabilities.
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    assert.equal(procTable.byName(procBName).caps.length, 0, "Procedure B should have no capabilities");
                }

                // Call the PushCap function to add a Write capability to
                // Procedure B.
                {
                    const functionSelectorHash = web3.sha3("PushCap(bytes24,uint256[])").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(64).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(4).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)
                        + web3.toHex(3).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(7).slice(2).padStart(32*2,0) // the type of the first (only) type, which is WRITE
                        + web3.toHex(0x8000).slice(2).padStart(32*2,0) // the location of the write cap
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the size of the write cap


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Check that Procedure B now has a Write capability.
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    assert.equal(procTable.byName(procBName).caps.length, 1, "Procedure B should have one capability");
                    assert.equal(procTable.byName(procBName).caps[0].type, 7, "Procedure B's first capabilty should be of type WRITE");
                }

                // Add a second write cap. We need to do this because Procedure
                // B expects the write cap to be in the second position.
                {
                    const functionSelectorHash = web3.sha3("PushCap(bytes24,uint256[])").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(64).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(4).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)
                        + web3.toHex(3).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(7).slice(2).padStart(32*2,0) // the type of the first (only) type, which is WRITE
                        + web3.toHex(0x8000).slice(2).padStart(32*2,0) // the location of the write cap
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the size of the write cap


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Check that Procedure B can successfully execute a write using this capability.
                {
                    const originalValue = await kernel.testGetter.call();
                    assert.equal(originalValue.toNumber(), 3, "The original value should be 3");
                    // "S()" is the function in Procedure B which executes a
                    // write.
                    const functionSelector = "S()";
                    const functionSelectorHash = web3.sha3(functionSelector).slice(2,10);
                    const inputData = web3.fromAscii(procBName.padEnd(24,"\0")) + functionSelectorHash;
                    await kernel.sendTransaction({data: inputData});
                    const newValue = await kernel.testGetter.call();
                    assert.equal(newValue.toNumber(), 4, "The value should be 4 after the execution");
                }


            })
        })
        describe('When insufficient caps given', function () {
            // Procedure A is contract which can add caps to another contract
            const contractA = Valid.SysCallTestCaps;
            // We use a contract which can test write calls, this is so we can
            // check that caps can be used properly.
            const contractB = Valid.SysCallTestWrite;

            it('Fail to give write cap to Procedure B', async function () {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A
                const procAName = "ProcedureA";
                const caps = [
                    new beakerlib.SetEntryCap()
                ];
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                {
                    await kernel.registerAnyProcedure(procAName, deployedEntryProc.address);
                    for (const cap of caps) {
                        await kernel.addCap(procAName, beakerlib.Cap.toInput([cap]))
                    }
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
                    assert.equal(valueX.toNumber(), 567, "should receive the correct test number");
                }

                // Install Procedure B
                const procBName = "ProcedureB";
                const deployedContractB = await testutils.deployedTrimmed(contractB);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procBName, deployedContractB.address);

                // Test that Procedure B executes correctly by calling the
                // testNum() function of that contract.
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procBName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 986, "should receive the correct test number");
                }

                // Check that initially Procedure B has no capabilities.
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    assert.equal(procTable.byName(procBName).caps.length, 0, "Procedure B should have no capabilities");
                }

                // Call the PushCap function to add a Write capability to
                // Procedure B.
                {
                    const functionSelectorHash = web3.sha3("PushCap(bytes24,uint256[])").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(64).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(4).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)
                        + web3.toHex(3).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(7).slice(2).padStart(32*2,0) // the type of the first (only) type, which is WRITE
                        + web3.toHex(0x8000).slice(2).padStart(32*2,0) // the location of the write cap
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the size of the write cap


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert(valueX.toNumber() != 0, "Should return a non-zero error code");
                }

                // Check that Procedure B still has no capabilities
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    assert.equal(procTable.byName(procBName).caps.length, 0, "Procedure B should have one capability");
                }

                // Add a second write cap. We need to do this because Procedure
                // B expects the write cap to be in the second position. This
                // should also fail due the lack of a capability.
                {
                    const functionSelectorHash = web3.sha3("PushCap(bytes24,uint256[])").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(64).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(4).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)
                        + web3.toHex(3).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(7).slice(2).padStart(32*2,0) // the type of the first (only) type, which is WRITE
                        + web3.toHex(0x8000).slice(2).padStart(32*2,0) // the location of the write cap
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the size of the write cap


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert(valueX.toNumber() != 0, "Should return a non-zero error code");
                }

                // Check that Procedure B cannot successfully execute a write using this capability.
                {
                    const originalValue = await kernel.testGetter.call();
                    assert.equal(originalValue.toNumber(), 3, "The original value should be 3");
                    // "S()" is the function in Procedure B which executes a
                    // write.
                    const functionSelector = "S()";
                    const functionSelectorHash = web3.sha3(functionSelector).slice(2,10);
                    const inputData = web3.fromAscii(procBName.padEnd(24,"\0")) + functionSelectorHash;
                    await kernel.sendTransaction({data: inputData});
                    const newValue = await kernel.testGetter.call();
                    assert.equal(newValue.toNumber(), 3, "The value should still be 3 after the execution");
                }


            })
        })
    })
    describe('Add 2 caps to Procedure B using Procedure A, then remove the first cap', function () {
        describe('When sufficient caps given', function () {
            // Procedure A is contract which can add caps to another contract
            const contractA = Valid.SysCallTestCaps;
            // We use a contract which can test write calls, this is so we can
            // check that caps can be used properly.
            const contractB = Valid.SysCallTestWrite;

            it('Give 2 write caps and log cap to Procedure B, then remove the second write cap', async function () {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A
                const procAName = "ProcedureA";
                const caps = [
                    new beakerlib.PushCap(),
                    new beakerlib.CapDeleteCap()
                ];
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                {
                    await kernel.registerAnyProcedure(procAName, deployedEntryProc.address);
                    for (const cap of caps) {
                        await kernel.addCap(procAName, beakerlib.Cap.toInput([cap]))
                    }
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
                    assert.equal(valueX.toNumber(), 567, "should receive the correct test number");
                }

                // Install Procedure B
                const procBName = "ProcedureB";
                const deployedContractB = await testutils.deployedTrimmed(contractB);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procBName, deployedContractB.address);

                // Test that Procedure B executes correctly by calling the
                // testNum() function of that contract.
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procBName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 986, "should receive the correct test number");
                }

                // Check that initially Procedure B has no capabilities.
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    assert.equal(procTable.byName(procBName).caps.length, 0, "Procedure B should have no capabilities");
                }

                // Call the PushCap function to add a Write capability to
                // Procedure B.
                {
                    const functionSelectorHash = web3.sha3("PushCap(bytes24,uint256[])").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(64).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(4).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)
                        + web3.toHex(3).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(7).slice(2).padStart(32*2,0) // the type of the first (only) type, which is WRITE
                        + web3.toHex(0x8000).slice(2).padStart(32*2,0) // the location of the write cap
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the size of the write cap


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Check that Procedure B now has a Write capability.
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    assert.equal(procTable.byName(procBName).caps.length, 1, "Procedure B should have one capability");
                    assert.equal(procTable.byName(procBName).caps[0].type, 7, "Procedure B's first capabilty should be of type WRITE");
                }

                // Add a second write cap. We need to do this because Procedure
                // B expects the write cap to be in the second position.
                {
                    const functionSelectorHash = web3.sha3("PushCap(bytes24,uint256[])").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(64).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(4).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)
                        + web3.toHex(3).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(7).slice(2).padStart(32*2,0) // the type of the first (only) type, which is WRITE
                        + web3.toHex(0x8000).slice(2).padStart(32*2,0) // the location of the write cap
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the size of the write cap


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Add a log cap.
                {
                    const functionSelectorHash = web3.sha3("PushCap(bytes24,uint256[])").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(64).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(2).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(9).slice(2).padStart(32*2,0) // the type of the first (only) type, which is LOG


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Check that Procedure B now has 2 write caps and a log cap
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    const expectedCaps = [
                        { type: '0x7', values: [ '0x8000', '0x1' ] },
                        { type: '0x7', values: [ '0x8000', '0x1' ] },
                        { type: '0x9', values: [] }
                    ];
                    assert.equal(procTable.byName(procBName).caps.length, expectedCaps.length, "Procedure B should three capabilities");
                    // Cycle through the caps and check that they are as expected
                    for (const i in procTable.byName(procBName).caps) {
                        assert.deepEqual(procTable.byName(procBName).caps[i], expectedCaps[i], `Capability #${i} is not as expected`);
                    }
                }

                // Check that Procedure B can successfully execute a write using this capability.
                {
                    const originalValue = await kernel.testGetter.call();
                    assert.equal(originalValue.toNumber(), 3, "The original value should be 3");
                    // "S()" is the function in Procedure B which executes a
                    // write.
                    const functionSelector = "S()";
                    const functionSelectorHash = web3.sha3(functionSelector).slice(2,10);
                    const inputData = web3.fromAscii(procBName.padEnd(24,"\0")) + functionSelectorHash;
                    await kernel.sendTransaction({data: inputData});
                    const newValue = await kernel.testGetter.call();
                    assert.equal(newValue.toNumber(), 4, "The value should be 4 after the execution");
                }

                // Delete the first cap
                {
                    const functionSelectorHash = web3.sha3("DeleteCap(bytes24,uint256)").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(0).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    const tx = await kernel.sendTransaction({data: manualInputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Check that the first cap has been successfully removed. This
                // means the capability will become null (0x0) not completely
                // removed.
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    const expectedCaps = [
                        { type: '0x0', values: [ '0x0', '0x0' ] },
                        { type: '0x7', values: [ '0x8000', '0x1' ] },
                        { type: '0x9', values: [] }
                    ];
                    assert.equal(procTable.byName(procBName).caps.length, expectedCaps.length, "Procedure B have the correct number of capabilities");
                    // Cycle through the caps and check that they are as expected
                    for (const i in procTable.byName(procBName).caps) {
                        assert.deepEqual(procTable.byName(procBName).caps[i], expectedCaps[i], `Capability #${i} is not as expected`);
                    }
                }

                // Check that Procedure B can still successfully write using this capability
                {
                    const originalValue = await kernel.testGetter.call();
                    assert.equal(originalValue.toNumber(), 4, "The original value should be 4");
                    // "S()" is the function in Procedure B which executes a
                    // write.
                    const functionSelector = "S()";
                    const functionSelectorHash = web3.sha3(functionSelector).slice(2,10);
                    const inputData = web3.fromAscii(procBName.padEnd(24,"\0")) + functionSelectorHash;
                    await kernel.sendTransaction({data: inputData});
                    const newValue = await kernel.testGetter.call();
                    assert.equal(newValue.toNumber(), 5, "The value should be 5 after the execution");
                }

                // Delete the third cap
                {
                    const functionSelectorHash = web3.sha3("DeleteCap(bytes24,uint256)").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(2).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    const tx = await kernel.sendTransaction({data: manualInputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Check that the third cap has been successfully removed. This
                // means the capability will become null (0x0) not completely
                // removed.
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    const expectedCaps = [
                        { type: '0x0', values: [ '0x0', '0x0' ] },
                        { type: '0x7', values: [ '0x8000', '0x1' ] },
                        { type: '0x0', values: [] }
                    ];
                    assert.equal(procTable.byName(procBName).caps.length, expectedCaps.length, "Procedure B have the correct number of capabilities");
                    // Cycle through the caps and check that they are as expected
                    for (const i in procTable.byName(procBName).caps) {
                        assert.deepEqual(procTable.byName(procBName).caps[i], expectedCaps[i], `Capability #${i} is not as expected`);
                    }
                }

                // Delete the second cap
                {
                    const functionSelectorHash = web3.sha3("DeleteCap(bytes24,uint256)").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    const tx = await kernel.sendTransaction({data: manualInputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Check that the third cap has been successfully removed. This
                // means the capability will become null (0x0) not completely
                // removed.
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    const expectedCaps = [
                        { type: '0x0', values: [ '0x0', '0x0' ] },
                        { type: '0x0', values: [ '0x0', '0x0' ] },
                        { type: '0x0', values: [] }
                    ];
                    assert.equal(procTable.byName(procBName).caps.length, expectedCaps.length, "Procedure B have the correct number of capabilities");
                    // Cycle through the caps and check that they are as expected
                    for (const i in procTable.byName(procBName).caps) {
                        assert.deepEqual(procTable.byName(procBName).caps[i], expectedCaps[i], `Capability #${i} is not as expected`);
                    }
                }

                // Check that Procedure B can no longer successfully write using
                // this capability
                {
                    const originalValue = await kernel.testGetter.call();
                    assert.equal(originalValue.toNumber(), 5, "The original value should be 5");
                    // "S()" is the function in Procedure B which executes a
                    // write.
                    const functionSelector = "S()";
                    const functionSelectorHash = web3.sha3(functionSelector).slice(2,10);
                    const inputData = web3.fromAscii(procBName.padEnd(24,"\0")) + functionSelectorHash;
                    await kernel.sendTransaction({data: inputData});
                    const newValue = await kernel.testGetter.call();
                    assert.equal(newValue.toNumber(), 5, "The value should still be 5 after the execution");
                }

            })
        })
        describe('Index out of range', function () {
            // Procedure A is contract which can add caps to another contract
            const contractA = Valid.SysCallTestCaps;
            // We use a contract which can test write calls, this is so we can
            // check that caps can be used properly.
            const contractB = Valid.SysCallTestWrite;

            it('Give 2 write caps and log cap to Procedure B, then remove the second write cap', async function () {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A
                const procAName = "ProcedureA";
                const caps = [
                    new beakerlib.PushCap(),
                    new beakerlib.CapDeleteCap()
                ];
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                {
                    await kernel.registerAnyProcedure(procAName, deployedEntryProc.address);
                    for (const cap of caps) {
                        await kernel.addCap(procAName, beakerlib.Cap.toInput([cap]))
                    }
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
                    assert.equal(valueX.toNumber(), 567, "should receive the correct test number");
                }

                // Install Procedure B
                const procBName = "ProcedureB";
                const deployedContractB = await testutils.deployedTrimmed(contractB);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procBName, deployedContractB.address);

                // Test that Procedure B executes correctly by calling the
                // testNum() function of that contract.
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procBName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 986, "should receive the correct test number");
                }

                // Check that initially Procedure B has no capabilities.
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    assert.equal(procTable.byName(procBName).caps.length, 0, "Procedure B should have no capabilities");
                }

                // Call the PushCap function to add a Write capability to
                // Procedure B.
                {
                    const functionSelectorHash = web3.sha3("PushCap(bytes24,uint256[])").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(64).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(4).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)
                        + web3.toHex(3).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(7).slice(2).padStart(32*2,0) // the type of the first (only) type, which is WRITE
                        + web3.toHex(0x8000).slice(2).padStart(32*2,0) // the location of the write cap
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the size of the write cap


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Check that Procedure B now has a Write capability.
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    assert.equal(procTable.byName(procBName).caps.length, 1, "Procedure B should have one capability");
                    assert.equal(procTable.byName(procBName).caps[0].type, 7, "Procedure B's first capabilty should be of type WRITE");
                }

                // Add a second write cap. We need to do this because Procedure
                // B expects the write cap to be in the second position.
                {
                    const functionSelectorHash = web3.sha3("PushCap(bytes24,uint256[])").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(64).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(4).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)
                        + web3.toHex(3).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(7).slice(2).padStart(32*2,0) // the type of the first (only) type, which is WRITE
                        + web3.toHex(0x8000).slice(2).padStart(32*2,0) // the location of the write cap
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the size of the write cap


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Add a log cap.
                {
                    const functionSelectorHash = web3.sha3("PushCap(bytes24,uint256[])").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(64).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(2).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(9).slice(2).padStart(32*2,0) // the type of the first (only) type, which is LOG


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Check that Procedure B now has 2 write caps and a log cap
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    const expectedCaps = [
                        { type: '0x7', values: [ '0x8000', '0x1' ] },
                        { type: '0x7', values: [ '0x8000', '0x1' ] },
                        { type: '0x9', values: [] }
                    ];
                    assert.equal(procTable.byName(procBName).caps.length, expectedCaps.length, "Procedure B should three capabilities");
                    // Cycle through the caps and check that they are as expected
                    for (const i in procTable.byName(procBName).caps) {
                        assert.deepEqual(procTable.byName(procBName).caps[i], expectedCaps[i], `Capability #${i} is not as expected`);
                    }
                }

                // Check that Procedure B can successfully execute a write using this capability.
                {
                    const originalValue = await kernel.testGetter.call();
                    assert.equal(originalValue.toNumber(), 3, "The original value should be 3");
                    // "S()" is the function in Procedure B which executes a
                    // write.
                    const functionSelector = "S()";
                    const functionSelectorHash = web3.sha3(functionSelector).slice(2,10);
                    const inputData = web3.fromAscii(procBName.padEnd(24,"\0")) + functionSelectorHash;
                    await kernel.sendTransaction({data: inputData});
                    const newValue = await kernel.testGetter.call();
                    assert.equal(newValue.toNumber(), 4, "The value should be 4 after the execution");
                }

                // Attempt to delete a capability out-of-bounds
                {
                    const functionSelectorHash = web3.sha3("DeleteCap(bytes24,uint256)").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(5).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    await kernel.sendTransaction({data: manualInputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert(valueX.toNumber() !== 0, "Should return a non-zero error code");
                }

                // Check that nothing has changed
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    const expectedCaps = [
                        { type: '0x7', values: [ '0x8000', '0x1' ] },
                        { type: '0x7', values: [ '0x8000', '0x1' ] },
                        { type: '0x9', values: [] }
                    ];
                    assert.equal(procTable.byName(procBName).caps.length, expectedCaps.length, "Procedure B should three capabilities");
                    // Cycle through the caps and check that they are as
                    // expected
                    for (const i in procTable.byName(procBName).caps) {
                        assert.deepEqual(procTable.byName(procBName).caps[i], expectedCaps[i], `Capability #${i} is not as expected`);
                    }
                }
            })
        })
        describe('When insufficient caps given', function () {
            // Procedure A is contract which can add caps to another contract
            const contractA = Valid.SysCallTestCaps;
            // We use a contract which can test write calls, this is so we can
            // check that caps can be used properly.
            const contractB = Valid.SysCallTestWrite;

            it('Give 2 write caps and log cap to Procedure B, then attempt to remove the second write cap', async function () {
                // Deploy the kernel
                const kernel = await Kernel.new();

                // Install the default entry procedure
                await testutils.installEntryProc(kernel);

                // Install Procedure A
                const procAName = "ProcedureA";
                const caps = [
                    new beakerlib.PushCap()
                ];
                const deployedEntryProc = await testutils.deployedTrimmed(contractA);
                // This uses a direct call to the kernel
                {
                    await kernel.registerAnyProcedure(procAName, deployedEntryProc.address);
                    for (const cap of caps) {
                        await kernel.addCap(procAName, beakerlib.Cap.toInput([cap]))
                    }
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
                    assert.equal(valueX.toNumber(), 567, "should receive the correct test number");
                }

                // Install Procedure B
                const procBName = "ProcedureB";
                const deployedContractB = await testutils.deployedTrimmed(contractB);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procBName, deployedContractB.address);

                // Test that Procedure B executes correctly by calling the
                // testNum() function of that contract.
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procBName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 986, "should receive the correct test number");
                }

                // Check that initially Procedure B has no capabilities.
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    assert.equal(procTable.byName(procBName).caps.length, 0, "Procedure B should have no capabilities");
                }

                // Call the PushCap function to add a Write capability to
                // Procedure B.
                {
                    const functionSelectorHash = web3.sha3("PushCap(bytes24,uint256[])").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(64).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(4).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)
                        + web3.toHex(3).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(7).slice(2).padStart(32*2,0) // the type of the first (only) type, which is WRITE
                        + web3.toHex(0x8000).slice(2).padStart(32*2,0) // the location of the write cap
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the size of the write cap


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Check that Procedure B now has a Write capability.
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    assert.equal(procTable.byName(procBName).caps.length, 1, "Procedure B should have one capability");
                    assert.equal(procTable.byName(procBName).caps[0].type, 7, "Procedure B's first capabilty should be of type WRITE");
                }

                // Add a second write cap. We need to do this because Procedure
                // B expects the write cap to be in the second position.
                {
                    const functionSelectorHash = web3.sha3("PushCap(bytes24,uint256[])").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(64).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(4).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)
                        + web3.toHex(3).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(7).slice(2).padStart(32*2,0) // the type of the first (only) type, which is WRITE
                        + web3.toHex(0x8000).slice(2).padStart(32*2,0) // the location of the write cap
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the size of the write cap


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Add a log cap.
                {
                    const functionSelectorHash = web3.sha3("PushCap(bytes24,uint256[])").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(64).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(2).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(9).slice(2).padStart(32*2,0) // the type of the first (only) type, which is LOG


                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    await kernel.sendTransaction({data: manualInputData});

                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Check that Procedure B now has 2 write caps and a log cap
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    const expectedCaps = [
                        { type: '0x7', values: [ '0x8000', '0x1' ] },
                        { type: '0x7', values: [ '0x8000', '0x1' ] },
                        { type: '0x9', values: [] }
                    ];
                    assert.equal(procTable.byName(procBName).caps.length, expectedCaps.length, "Procedure B should three capabilities");
                    // Cycle through the caps and check that they are as expected
                    for (const i in procTable.byName(procBName).caps) {
                        assert.deepEqual(procTable.byName(procBName).caps[i], expectedCaps[i], `Capability #${i} is not as expected`);
                    }
                }

                // Check that Procedure B can successfully execute a write using this capability.
                {
                    const originalValue = await kernel.testGetter.call();
                    assert.equal(originalValue.toNumber(), 3, "The original value should be 3");
                    // "S()" is the function in Procedure B which executes a
                    // write.
                    const functionSelector = "S()";
                    const functionSelectorHash = web3.sha3(functionSelector).slice(2,10);
                    const inputData = web3.fromAscii(procBName.padEnd(24,"\0")) + functionSelectorHash;
                    await kernel.sendTransaction({data: inputData});
                    const newValue = await kernel.testGetter.call();
                    assert.equal(newValue.toNumber(), 4, "The value should be 4 after the execution");
                }

                // Delete the first cap
                {
                    const functionSelectorHash = web3.sha3("DeleteCap(bytes24,uint256)").slice(2,10);
                    const manualInputData = web3.fromAscii(procAName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name of the procedure add the capability to (32 bytes)
                        + web3.toHex(0).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)

                    const valueXRaw = await web3.eth.call({to: kernel.address, data: manualInputData});
                    const tx = await kernel.sendTransaction({data: manualInputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    assert.equal(valueX.toNumber(), 0, "Should return a zero error code");
                }

                // Check that the first cap has not been removed.
                {
                    const procTableData = await kernel.returnProcedureTable.call();
                    const procTable = beakerlib.ProcedureTable.parse(procTableData);
                    const expectedCaps = [
                        { type: '0x7', values: [ '0x8000', '0x1' ] },
                        { type: '0x7', values: [ '0x8000', '0x1' ] },
                        { type: '0x9', values: [] }
                    ];
                    assert.equal(procTable.byName(procBName).caps.length, expectedCaps.length, "Procedure B have the correct number of capabilities");
                    // Cycle through the caps and check that they are as expected
                    for (const i in procTable.byName(procBName).caps) {
                        assert.deepEqual(procTable.byName(procBName).caps[i], expectedCaps[i], `Capability #${i} is not as expected`);
                    }
                }
            })
        })
    })
})
