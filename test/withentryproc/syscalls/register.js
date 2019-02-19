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
    SysCallTestProcRegister: artifacts.require('test/valid/SysCallTestProcRegister.sol'),
    BasicEntryProcedure: artifacts.require('BasicEntryProcedure.sol'),
}

const TestWrite = artifacts.require('test/TestWrite.sol');

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

contract('Kernel with entry procedure', function (accounts) {
    describe('Register capability', function () {
        const procName = "SysCallTestProcRegister";
        const contract = Valid.SysCallTestProcRegister;

        describe('A(bytes24,address) - register a procedure', function () {
            const testProcName = "Adder";
            const testContract = Valid.Adder;
            const functionSpec = "A(bytes24,address)";
            it('A(bytes24,address) should succeed when given cap', async function () {
                const kernel = await Kernel.new();
                // console.log(`kernel: ${kernel.address}`);

                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures1:", procedures1);

                await testutils.installEntryProc(kernel);

                const procedures2Raw = await kernel.listProcedures.call();
                const procedures2 = procedures2Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures2:", procedures2);

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.RegisterCap();
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // for (const log of tx1.receipt.logs) {
                //     // console.log(`${log.topics} - ${log.data}`);
                //     console.log(`${log.topics} - ${log.data}`);
                //     try {
                //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                //     } catch(e) {
                //         // console.log(`${log.topics} - ${log.data}`);
                //         console.log("non-ascii");
                //     }
                // }
                // This is the procedure that will be registered
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // console.log(deployedTestContract.address)
                const procedures3Raw = await kernel.listProcedures.call();
                const procedures3 = procedures3Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures3:", procedures3);
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 392, "should receive the correct test number");
                }

                {
                    // console.log(deployedTestContract.address.slice(2))
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    // here we use padStart because 'address' is like a number, not bytes
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash + web3.fromAscii(testProcName.padEnd(32*2,"\0")).slice(2) + deployedTestContract.address.slice(2).padStart(32*2,0);
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // for (const log of tx3.receipt.logs) {
                    //     // console.log(`${log.topics} - ${log.data}`);
                    //     console.log(`${log.topics} - ${log.data}`);
                    //     try {
                    //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                    //     } catch(e) {
                    //         // console.log(`${log.topics} - ${log.data}`);
                    //         console.log("non-ascii");
                    //     }
                    // }
                    // console.log(valueX.toNumber())
                    assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");
                }

                const procedures4Raw = await kernel.listProcedures.call();
                const procedures4 = procedures4Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures4:", procedures4);
                assert(procedures4.includes(testProcName), "The correct name should be in the procedure table");
                assert.strictEqual(procedures4.length, (procedures3.length+1), "The number of procedures should have increased by 1");
            })
            it('A(bytes24,address) should fail when not given cap', async function () {
                const kernel = await Kernel.new();
                // console.log(`kernel: ${kernel.address}`);

                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures1:", procedures1);

                await testutils.installEntryProc(kernel);

                const procedures2Raw = await kernel.listProcedures.call();
                const procedures2 = procedures2Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures2:", procedures2);

                const deployedContract = await testutils.deployedTrimmed(contract);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, []);
                // for (const log of tx1.receipt.logs) {
                //     // console.log(`${log.topics} - ${log.data}`);
                //     console.log(`${log.topics} - ${log.data}`);
                //     try {
                //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                //     } catch(e) {
                //         // console.log(`${log.topics} - ${log.data}`);
                //         console.log("non-ascii");
                //     }
                // }
                // This is the procedure that will be registered
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // console.log(deployedTestContract.address)
                // const tx2 = await kernel.registerProcedure(testProcName, deployedTestContract.address, []);
                // const tx2 = await kernel.registerProcedure("testtesttesttesttesttesa", "0xf958a87ec617211109ec02846ec0df996532b104", []);
                const procedures3Raw = await kernel.listProcedures.call();
                const procedures3 = procedures3Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures3:", procedures3);
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 392, "should receive the correct test number");
                }

                {
                    // console.log(deployedTestContract.address.slice(2))
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    // here we use padStart because 'address' is like a number, not bytes
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash + web3.fromAscii(testProcName.padEnd(32*2,"\0")).slice(2) + deployedTestContract.address.slice(2).padStart(32*2,0);
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // for (const log of tx3.receipt.logs) {
                    //     // console.log(`${log.topics} - ${log.data}`);
                    //     console.log(`${log.topics} - ${log.data}`);
                    //     try {
                    //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                    //     } catch(e) {
                    //         // console.log(`${log.topics} - ${log.data}`);
                    //         console.log("non-ascii");
                    //     }
                    // }
                    // console.log(valueX.toNumber())
                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }

                const procedures4Raw = await kernel.listProcedures.call();
                const procedures4 = procedures4Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures4:", procedures4);
                assert(!procedures4.includes(testProcName), "The correct name should not be in the procedure table");
                assert.strictEqual(procedures4.length, procedures3.length, "The number of procedures should not have increased");
            })
            it('A(bytes24,address) should fail when given the wrong cap', async function () {
                const kernel = await Kernel.new();
                // console.log(`kernel: ${kernel.address}`);

                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures1:", procedures1);

                await testutils.installEntryProc(kernel);

                const procedures2Raw = await kernel.listProcedures.call();
                const procedures2 = procedures2Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures2:", procedures2);

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // for (const log of tx1.receipt.logs) {
                //     // console.log(`${log.topics} - ${log.data}`);
                //     console.log(`${log.topics} - ${log.data}`);
                //     try {
                //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                //     } catch(e) {
                //         // console.log(`${log.topics} - ${log.data}`);
                //         console.log("non-ascii");
                //     }
                // }
                // This is the procedure that will be registered
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // console.log(deployedTestContract.address)
                // const tx2 = await kernel.registerProcedure(testProcName, deployedTestContract.address, []);
                // const tx2 = await kernel.registerProcedure("testtesttesttesttesttesa", "0xf958a87ec617211109ec02846ec0df996532b104", []);
                const procedures3Raw = await kernel.listProcedures.call();
                const procedures3 = procedures3Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures3:", procedures3);
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 392, "should receive the correct test number");
                }

                {
                    // console.log(deployedTestContract.address.slice(2))
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    // here we use padStart because 'address' is like a number, not bytes
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash + web3.fromAscii(testProcName.padEnd(32*2,"\0")).slice(2) + deployedTestContract.address.slice(2).padStart(32*2,0);
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // for (const log of tx3.receipt.logs) {
                    //     // console.log(`${log.topics} - ${log.data}`);
                    //     console.log(`${log.topics} - ${log.data}`);
                    //     try {
                    //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                    //     } catch(e) {
                    //         // console.log(`${log.topics} - ${log.data}`);
                    //         console.log("non-ascii");
                    //     }
                    // }
                    // console.log(valueX.toNumber())
                    assert.equal(valueX.toNumber(), 4455, "should succeed with zero errcode the first time");
                }

                const procedures4Raw = await kernel.listProcedures.call();
                const procedures4 = procedures4Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures4:", procedures4);
                assert(!procedures4.includes(testProcName), "The correct name should not be in the procedure table");
                assert.strictEqual(procedures4.length, procedures3.length, "The number of procedures should not have increased");
            })
        })
        describe('B(bytes24 procName, address procAddress, uint256[] caps) - register a procedure', function () {
            const testProcName = "Adder";
            const testContract = Valid.Adder;
            const functionSpec = "B(bytes24,address,uint256[])";
            // console.log(contract);
            it('B(bytes24 procName, address procAddress, uint256[] caps) should succeed when given cap (the registered contract will not have caps)', async function () {
                const kernel = await Kernel.new();
                // console.log(`kernel: ${kernel.address}`);

                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures1:", procedures1);

                await testutils.installEntryProc(kernel);

                const procedures2Raw = await kernel.listProcedures.call();
                const procedures2 = procedures2Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures2:", procedures2);

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.RegisterCap();
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // for (const log of tx1.receipt.logs) {
                //     // console.log(`${log.topics} - ${log.data}`);
                //     console.log(`${log.topics} - ${log.data}`);
                //     try {
                //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                //     } catch(e) {
                //         // console.log(`${log.topics} - ${log.data}`);
                //         console.log("non-ascii");
                //     }
                // }
                // This is the procedure that will be registered
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // console.log("deployedContractAddress:", deployedTestContract.address);
                const procedures3Raw = await kernel.listProcedures.call();
                const procedures3 = procedures3Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures3:", procedures3);
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 392, "should receive the correct test number");
                }

                {
                    // console.log(deployedTestContract.address.slice(2))
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    // here we use padStart because 'address' is like a number, not bytes
                    // this inputData is custom built, it can be deleted if
                    // necessary, but shows the underlying input data
                    const manualInputData = web3.fromAscii(procName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(testProcName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name argument for register (32 bytes)
                        + deployedTestContract.address.slice(2).padStart(32*2,0) // the address argument for register (32 bytes)
                        + web3.toHex(124).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(0).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of zero (32 bytes)
                    // console.log(manualInputData)
                    // when using web3 1.0 this will be good
                    // try {
                    //     console.log(deployedContract.methods.B(testProcName,deployedTestContract.address,[]).data)
                    // } catch (e) {
                    //     console.log(e)
                    // }
                    const inputData = manualInputData;
                    // assert.strictEqual(inputData,)
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // for (const log of tx3.receipt.logs) {
                    //     // console.log(`${log.topics} - ${log.data}`);
                    //     console.log(`${log.topics} - ${log.data}`);
                    //     try {
                    //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                    //     } catch(e) {
                    //         // console.log(`${log.topics} - ${log.data}`);
                    //         console.log("non-ascii");
                    //     }
                    // }
                    // console.log(valueX.toNumber())
                    assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");
                }

                const procedures4Raw = await kernel.listProcedures.call();
                const procedures4 = procedures4Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures4:", procedures4);
                assert(procedures4.includes(testProcName), "The correct name should be in the procedure table");
                assert.strictEqual(procedures4.length, (procedures3.length+1), "The number of procedures should have increased by 1");

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);
                // console.log("Kernel Address:", kernel.address)
                // console.log(beakerlib.ProcedureTable.stringify(procTable));
            })
            it('B(bytes24 procName, address procAddress, uint256[] caps) should succeed when given cap (the registered contract will have 1 cap)', async function () {
                const kernel = await Kernel.new();
                // console.log(`kernel: ${kernel.address}`);

                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures1:", procedures1);

                await testutils.installEntryProc(kernel);

                const procedures2Raw = await kernel.listProcedures.call();
                const procedures2 = procedures2Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures2:", procedures2);

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.RegisterCap();
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // for (const log of tx1.receipt.logs) {
                //     // console.log(`${log.topics} - ${log.data}`);
                //     console.log(`${log.topics} - ${log.data}`);
                //     try {
                //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                //     } catch(e) {
                //         // console.log(`${log.topics} - ${log.data}`);
                //         console.log("non-ascii");
                //     }
                // }
                // This is the procedure that will be registered
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // console.log("deployedContractAddress:", deployedTestContract.address);
                const procedures3Raw = await kernel.listProcedures.call();
                const procedures3 = procedures3Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures3:", procedures3);
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 392, "should receive the correct test number");
                }

                {
                    // console.log(deployedTestContract.address.slice(2))
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    // here we use padStart because 'address' is like a number, not bytes
                    // this inputData is custom built, it can be deleted if
                    // necessary, but shows the underlying input data
                    const manualInputData = web3.fromAscii(procName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(testProcName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name argument for register (32 bytes)
                        + deployedTestContract.address.slice(2).padStart(32*2,0) // the address argument for register (32 bytes)
                        + web3.toHex(96).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)
                        + web3.toHex(2).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(9).slice(2).padStart(32*2,0) // the type of the first (only) type, which is "log any"
                    // console.log(manualInputData)
                    // // when using web3 1.0 this will be good
                    // try {
                    //     console.log(deployedContract.methods.B(testProcName,deployedTestContract.address,[]).data)
                    // } catch (e) {
                    //     console.log(e)
                    // }
                    const inputData = manualInputData;
                    // assert.strictEqual(inputData,)
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // for (const log of tx3.receipt.logs) {
                    //     // console.log(`${log.topics} - ${log.data}`);
                    //     console.log(`${log.topics} - ${log.data}`);
                    //     try {
                    //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                    //     } catch(e) {
                    //         // console.log(`${log.topics} - ${log.data}`);
                    //         console.log("non-ascii");
                    //     }
                    // }
                    // console.log(valueX.toNumber())
                    assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");
                }

                const procedures4Raw = await kernel.listProcedures.call();
                const procedures4 = procedures4Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures4:", procedures4);
                assert(procedures4.includes(testProcName), "The correct name should be in the procedure table");
                assert.strictEqual(procedures4.length, (procedures3.length+1), "The number of procedures should have increased by 1");

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);
                // console.log("Kernel Address:", kernel.address)
                // console.log(beakerlib.ProcedureTable.stringify(procTable));
                // console.log(procTable)
                const encodedName = web3.toHex(testProcName.padEnd(24,'\0'));
                // console.log(encodedName)
                // console.log(procTable.procedures[encodedName])
                assert.equal(1,procTable.procedures[encodedName].caps.length, "The procedure should have 1 cap");
            })
            it('B(bytes24 procName, address procAddress, uint256[] caps) should succeed when given cap (the registered contract will have 2 caps)', async function () {
                const kernel = await Kernel.new();
                // console.log(`kernel: ${kernel.address}`);

                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures1:", procedures1);

                await testutils.installEntryProc(kernel);

                const procedures2Raw = await kernel.listProcedures.call();
                const procedures2 = procedures2Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures2:", procedures2);

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.RegisterCap();
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // for (const log of tx1.receipt.logs) {
                //     // console.log(`${log.topics} - ${log.data}`);
                //     console.log(`${log.topics} - ${log.data}`);
                //     try {
                //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                //     } catch(e) {
                //         // console.log(`${log.topics} - ${log.data}`);
                //         console.log("non-ascii");
                //     }
                // }
                // This is the procedure that will be registered
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // console.log("deployedContractAddress:", deployedTestContract.address);
                const procedures3Raw = await kernel.listProcedures.call();
                const procedures3 = procedures3Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures3:", procedures3);
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 392, "should receive the correct test number");
                }

                {
                    // console.log(deployedTestContract.address.slice(2))
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    // here we use padStart because 'address' is like a number, not bytes
                    // this inputData is custom built, it can be deleted if
                    // necessary, but shows the underlying input data
                    const manualInputData = web3.fromAscii(procName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(testProcName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name argument for register (32 bytes)
                        + deployedTestContract.address.slice(2).padStart(32*2,0) // the address argument for register (32 bytes)
                        + web3.toHex(96).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)

                        + web3.toHex(6).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)

                        + web3.toHex(1).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(9).slice(2).padStart(32*2,0) // the type of the first (only) type, which is "log any"

                        + web3.toHex(3).slice(2).padStart(32*2,0) // the length of the second cap, which is 1
                        + web3.toHex(7).slice(2).padStart(32*2,0) // the type of the second cap, which is "write"
                        + web3.toHex(0x8000).slice(2).padStart(32*2,0) // the address of the wrote
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the number of additional keys

                    // console.log(manualInputData)
                    // // when using web3 1.0 this will be good
                    // try {
                    //     console.log(deployedContract.methods.B(testProcName,deployedTestContract.address,[]).data)
                    // } catch (e) {
                    //     console.log(e)
                    // }
                    const inputData = manualInputData;
                    // assert.strictEqual(inputData,)
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // for (const log of tx3.receipt.logs) {
                    //     // console.log(`${log.topics} - ${log.data}`);
                    //     console.log(`${log.topics} - ${log.data}`);
                    //     try {
                    //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                    //     } catch(e) {
                    //         // console.log(`${log.topics} - ${log.data}`);
                    //         console.log("non-ascii");
                    //     }
                    // }
                    // console.log(valueX.toNumber())
                    assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");
                }

                const procedures4Raw = await kernel.listProcedures.call();
                const procedures4 = procedures4Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures4:", procedures4);
                assert(procedures4.includes(testProcName), "The correct name should be in the procedure table");
                assert.strictEqual(procedures4.length, (procedures3.length+1), "The number of procedures should have increased by 1");

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);
                console.log("Kernel Address:", kernel.address)
                // console.log(beakerlib.ProcedureTable.stringify(procTable));
                const encodedName = web3.toHex(testProcName.padEnd(24,'\0'));
                assert.equal(2,procTable.procedures[encodedName].caps.length, "The procedure should have 2 caps");

                assert.equal(0x9,procTable.procedures[encodedName].caps[0].type, "The first cap should be of type 0x9");
                assert.equal(0,procTable.procedures[encodedName].caps[0].values.length, "The first cap should have no values associated with it");

                assert.equal(0x7,procTable.procedures[encodedName].caps[1].type, "The second cap should be of type 0x7");
                assert.equal(2,procTable.procedures[encodedName].caps[1].values.length, "The second cap should have 2 values associated with it");
                assert.equal(0x8000,procTable.procedures[encodedName].caps[1].values[0], "The first value of the second cap should be 0x8000");
                assert.equal(1,procTable.procedures[encodedName].caps[1].values[1], "The second value of the second cap should be 1");
            })
            it('B(bytes24 procName, address procAddress, uint256[] caps) should fail when not given cap (the registered contract tries to have 2 caps)', async function () {
                const kernel = await Kernel.new();
                // console.log(`kernel: ${kernel.address}`);

                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures1:", procedures1);

                await testutils.installEntryProc(kernel);

                const procedures2Raw = await kernel.listProcedures.call();
                const procedures2 = procedures2Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures2:", procedures2);

                const deployedContract = await testutils.deployedTrimmed(contract);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, []);
                // for (const log of tx1.receipt.logs) {
                //     // console.log(`${log.topics} - ${log.data}`);
                //     console.log(`${log.topics} - ${log.data}`);
                //     try {
                //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                //     } catch(e) {
                //         // console.log(`${log.topics} - ${log.data}`);
                //         console.log("non-ascii");
                //     }
                // }
                // This is the procedure that will be registered
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // console.log("deployedContractAddress:", deployedTestContract.address);
                const procedures3Raw = await kernel.listProcedures.call();
                const procedures3 = procedures3Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures3:", procedures3);
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 392, "should receive the correct test number");
                }

                {
                    // console.log(deployedTestContract.address.slice(2))
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    // here we use padStart because 'address' is like a number, not bytes
                    // this inputData is custom built, it can be deleted if
                    // necessary, but shows the underlying input data
                    const manualInputData = web3.fromAscii(procName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(testProcName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name argument for register (32 bytes)
                        + deployedTestContract.address.slice(2).padStart(32*2,0) // the address argument for register (32 bytes)
                        + web3.toHex(96).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)

                        + web3.toHex(6).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)

                        + web3.toHex(1).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(9).slice(2).padStart(32*2,0) // the type of the first (only) type, which is "log any"

                        + web3.toHex(3).slice(2).padStart(32*2,0) // the length of the second cap, which is 1
                        + web3.toHex(7).slice(2).padStart(32*2,0) // the type of the second cap, which is "write"
                        + web3.toHex(0x8000).slice(2).padStart(32*2,0) // the address of the wrote
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the number of additional keys

                    // console.log(manualInputData)
                    // // when using web3 1.0 this will be good
                    // try {
                    //     console.log(deployedContract.methods.B(testProcName,deployedTestContract.address,[]).data)
                    // } catch (e) {
                    //     console.log(e)
                    // }
                    const inputData = manualInputData;
                    // assert.strictEqual(inputData,)
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // for (const log of tx3.receipt.logs) {
                    //     // console.log(`${log.topics} - ${log.data}`);
                    //     console.log(`${log.topics} - ${log.data}`);
                    //     try {
                    //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                    //     } catch(e) {
                    //         // console.log(`${log.topics} - ${log.data}`);
                    //         console.log("non-ascii");
                    //     }
                    // }
                    // console.log(valueX.toNumber())
                    assert.equal(valueX.toNumber(), 5599, "should succeed with zero errcode the first time");
                }

                const procedures4Raw = await kernel.listProcedures.call();
                const procedures4 = procedures4Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures4:", procedures4);
                assert(!procedures4.includes(testProcName), "The name should notbe in the procedure table");
                assert.strictEqual(procedures4.length, procedures3.length, "The number of procedures should have remained the same");

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);
                // console.log("Kernel Address:", kernel.address)
                // console.log(beakerlib.ProcedureTable.stringify(procTable));
                const encodedName = web3.toHex(testProcName.padEnd(24,'\0'));
                assert(procTable.procedures[encodedName] === undefined, "The procedure should not be present in the table");
            })
            it('B(bytes24 procName, address procAddress, uint256[] caps) should fail when given the wrong cap (the registered contract tries to have 2 caps)', async function () {
                const kernel = await Kernel.new();
                // console.log(`kernel: ${kernel.address}`);

                const procedures1Raw = await kernel.listProcedures.call();
                const procedures1 = procedures1Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures1:", procedures1);

                await testutils.installEntryProc(kernel);

                const procedures2Raw = await kernel.listProcedures.call();
                const procedures2 = procedures2Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures2:", procedures2);

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                // This is the procedure that will do the registering
                // this currently requires Any because it uses logging for testing
                const tx1 = await kernel.registerAnyProcedure(procName, deployedContract.address, capArray);
                // for (const log of tx1.receipt.logs) {
                //     // console.log(`${log.topics} - ${log.data}`);
                //     console.log(`${log.topics} - ${log.data}`);
                //     try {
                //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                //     } catch(e) {
                //         // console.log(`${log.topics} - ${log.data}`);
                //         console.log("non-ascii");
                //     }
                // }
                // This is the procedure that will be registered
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // console.log("deployedContractAddress:", deployedTestContract.address);
                const procedures3Raw = await kernel.listProcedures.call();
                const procedures3 = procedures3Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures3:", procedures3);
                {
                    const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
                    const inputData = web3.fromAscii(procName.padEnd(24,"\0")) + functionSelectorHash;
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // we execute a test function to ensure the procedure is
                    // functioning properly
                    assert.equal(valueX.toNumber(), 392, "should receive the correct test number");
                }

                {
                    // console.log(deployedTestContract.address.slice(2))
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
                    // here we use padStart because 'address' is like a number, not bytes
                    // this inputData is custom built, it can be deleted if
                    // necessary, but shows the underlying input data
                    const manualInputData = web3.fromAscii(procName.padEnd(24,"\0")) // the name of the procedure to call (24 bytes)
                        + functionSelectorHash // the function selector hash (4 bytes)
                        + web3.fromAscii(testProcName.padEnd(24,"\0")).slice(2).padEnd(32*2,0) // the name argument for register (32 bytes)
                        + deployedTestContract.address.slice(2).padStart(32*2,0) // the address argument for register (32 bytes)
                        + web3.toHex(96).slice(2).padStart(32*2,0) // the offset for the start of caps data (32 bytes)

                        + web3.toHex(6).slice(2).padStart(32*2,0) // the caps data, which is currently just a length of 2 (32 bytes)

                        + web3.toHex(1).slice(2).padStart(32*2,0) // the length of the first (only) cap, which is 1
                        + web3.toHex(9).slice(2).padStart(32*2,0) // the type of the first (only) type, which is "log any"

                        + web3.toHex(3).slice(2).padStart(32*2,0) // the length of the second cap, which is 1
                        + web3.toHex(7).slice(2).padStart(32*2,0) // the type of the second cap, which is "write"
                        + web3.toHex(0x8000).slice(2).padStart(32*2,0) // the address of the wrote
                        + web3.toHex(1).slice(2).padStart(32*2,0) // the number of additional keys

                    // console.log(manualInputData)
                    // // when using web3 1.0 this will be good
                    // try {
                    //     console.log(deployedContract.methods.B(testProcName,deployedTestContract.address,[]).data)
                    // } catch (e) {
                    //     console.log(e)
                    // }
                    const inputData = manualInputData;
                    // assert.strictEqual(inputData,)
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    const tx3 = await kernel.sendTransaction({data: inputData});
                    const valueX = web3.toBigNumber(valueXRaw);
                    // for (const log of tx3.receipt.logs) {
                    //     // console.log(`${log.topics} - ${log.data}`);
                    //     console.log(`${log.topics} - ${log.data}`);
                    //     try {
                    //         console.log(`${log.topics.map(web3.toAscii)} - ${web3.toAscii(log.data)}`);
                    //     } catch(e) {
                    //         // console.log(`${log.topics} - ${log.data}`);
                    //         console.log("non-ascii");
                    //     }
                    // }
                    // console.log(valueX.toNumber())
                    assert.equal(valueX.toNumber(), 5599, "should succeed with zero errcode the first time");
                }

                const procedures4Raw = await kernel.listProcedures.call();
                const procedures4 = procedures4Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures4:", procedures4);
                assert(!procedures4.includes(testProcName), "The name should not be in the procedure table");
                assert.strictEqual(procedures4.length, procedures3.length, "The number of procedures should not have changed");

                const procTableData = await kernel.returnProcedureTable.call();
                const procTable = beakerlib.ProcedureTable.parse(procTableData);
                // console.log("Kernel Address:", kernel.address)
                // console.log(beakerlib.ProcedureTable.stringify(procTable));
                const encodedName = web3.toHex(testProcName.padEnd(24,'\0'));
                assert(procTable.procedures[encodedName] === undefined, "The procedure should not be present in the table");
            })
        })
    })
})