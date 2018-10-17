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
    SysCallTestCreate: artifacts.require('test/valid/SysCallTestCreate.sol'),
    BasicEntryProcedure: artifacts.require('BasicEntryProcedure.sol'),
}

const TestWrite = artifacts.require('test/TestWrite.sol');

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

contract('Kernel with entry procedure', function (accounts) {
    describe('Register capability', function () {
        const procName = "SysCallTestCreate";
        const contract = Valid.SysCallTestCreate;

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
                const cap2 = new beakerlib.RegisterCap([]);
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
                    assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");
                }

                const procedures4Raw = await kernel.listProcedures.call();
                const procedures4 = procedures4Raw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                // console.log("procedures4:", procedures4);
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
            })
        })
    })
})