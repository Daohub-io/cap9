const debug = require('debug')
const assert = require('assert')

const Kernel = artifacts.require('./Kernel.sol')
const abi = require('ethereumjs-abi')

const beakerlib = require("../../../beakerlib");
const testutils = require("../../testutils.js");

const rpc = require('node-json-rpc');

const options = {
  // int port of rpc server, default 5080 for http or 5433 for https
  port: 8545,
  // string domain name or ip of rpc server, default '127.0.0.1'
  host: '127.0.0.1',
  // string with default path, default '/'
  path: '/',
  // boolean false to turn rpc checks off, default true
  strict: true
};

// Create a server object with options
const client = new rpc.Client(options);

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
}

const TestWrite = artifacts.require('test/TestWrite.sol');

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

contract('Kernel without entry procedure', function (accounts) {
    describe('Call capability', function () {
        const procName = "SysCallTestCall";
        const contract = Valid.SysCallTestCall;
        const bytecode = Valid.SysCallTestCall.bytecode;

        describe('A() - call procedure which needs no caps', function () {
            const testProcName = "TestWrite";
            const testBytecode = TestWrite.bytecode;
            const testContract = TestWrite;
            const functionSpec = "A()";
            it('A() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();
                const addresses = new Map();

                addresses.set(kernel.address, "kernel");
                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");
                console.log("originalValue", originalValue)
                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                addresses.set(deployedContract.address, procName);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                addresses.set(deployedTestContract.address, testProcName);
                // This is the procedure that will do the calling
                const n = web3.utils.asciiToHex(procName.padEnd(24, "\0"))
                console.log("n", n);
                let tx1;
                try {
                    // this transaction is failing
                    tx1 = await kernel.registerProcedure(n, deployedContract.address, capArray);
                } catch (e) {
                    console.log(e)
                    tx1=e;
                }
                console.log(tx1)
                const trace = await new Promise((resolve,reject) => {
                    const input = {"jsonrpc": "2.0", "method": "trace_replayTransaction", "params": [tx1.receipt.transactionHash,["trace"]], "id": 0};
                    console.log(input)
                    client.call(
                        input,
                        function (err, res) {
                            if (err) {
                                reject(err)
                            } else {
                                resolve(res);
                            }
                        }
                    );
                });

                function printTrace(addresses, result) {
                    console.log(`output: ${result.output}`);
                    console.log(`stateDiff: ${result.stateDiff}`);
                    for (const obj of result.trace) {
                        // console.log(result.trace[i]);
                        let str = "";
                        for (const n of obj.traceAddress) {
                            str += "  ";
                        }
                        const name = addresses.get(obj.action.to);
                        if (name) {
                            str += name;
                        } else {
                            str += obj.action.to;
                        }
                        str += ` (${obj.action.callType})`;
                        if (obj.error) {
                            str += " " + obj.error
                        }
                        console.log(str);
                        // console.log(result.trace[i]);
                    }
                    return trace;
                }
                console.log(trace.result.trace);
                printTrace(addresses, trace.result);
                console.log("kernel.address", kernel.address)
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                // console.log(web3.toHex(valueX))
                // try {
                //     console.log(web3.toAscii(web3.toHex(valueX)))
                // } catch (e) {

                // }
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                // console.log(tx)
                // for (const log of tx.receipt.logs) {
                //     if (log.topics.length > 0) {
                //         console.log(`Log: ${web3.toAscii(log.topics[0])} - ${log.data} - ${web3.toAscii(log.data)}`);
                //     } else {
                //         console.log(`Log: ${log.topics[0]} - ${web3.toAscii(log.data)} - ${log.data}`);
                //     }
                // }
                assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),356, "new value should be 356");
            })
            it('A() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
            it('A() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
            it('A() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(["another-proc", testProcName]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),356, "new value should be 356");
            })
            it('A() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([procName+"abc"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, []);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
        })
        describe('B() - without data', function () {
            const testProcName = "SysCallTest";
            const testContract = Valid.SysCallTest;
            const testBytecode = Valid.SysCallTest.bytecode;
            const functionSpec = "B()";
            it('B() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();
                const addresses = new Map();
                addresses.set(kernel.address, "kernel");

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);


                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                addresses.set(deployedContract.address, procName);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap2, cap1]));
                addresses.set(deployedTestContract.address, testProcName);

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                console.log("calling")
                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                console.log("sending")
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32, {gas:8721975});
                console.log(tx)
                console.log(tx.receipt.logs)

                const trace = await new Promise((resolve,reject) => {
                    const input = {"jsonrpc": "2.0", "method": "trace_replayTransaction", "params": [tx.receipt.transactionHash,["trace"]], "id": 0};
                    console.log(input)
                    client.call(
                        input,
                        function (err, res) {
                            if (err) {
                                reject(err)
                            } else {
                                resolve(res);
                            }
                        }
                    );
                });

                function printTrace(addresses, result) {
                    console.log(`output: ${result.output}`);
                    console.log(`stateDiff: ${result.stateDiff}`);
                    for (const obj of result.trace) {
                        // console.log(result.trace[i]);
                        let str = "";
                        for (const n of obj.traceAddress) {
                            str += "  ";
                        }
                        const name = addresses.get(obj.action.to);
                        if (name) {
                            str += name;
                        } else {
                            str += obj.action.to;
                        }
                        str += ` (${obj.action.callType})`;
                        if (obj.error) {
                            str += " " + obj.error
                        }
                        console.log(str);
                        // console.log(result.trace[i]);
                    }
                    return trace;
                }
                console.log(trace.result.trace);
                printTrace(addresses, trace.result);
                assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),356, "new value should be 356");
            })
            it('B() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);


                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
            it('B() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
            it('B() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(["another-proc", testProcName]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap2, cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),356, "new value should be 356");
            })
            it('B() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([procName+"abc"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
        })
        describe('C() - with data (function selector)', function () {
            const testProcName = "SysCallTest";
            const testBytecode = Valid.SysCallTest.bytecode;
            const testContract = Valid.SysCallTest;
            const functionSpec = "C()";
            it('C() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap2, cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);

                assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),(originalValue.toNumber() + 1), `new value should be ${originalValue.toNumber()+1}`);
            })
            it('C() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),originalValue.toNumber(), `new value should still be ${originalValue.toNumber()}`);
            })
            it('C() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),originalValue.toNumber(), `new value should still be ${originalValue.toNumber()}`);
            })
            it('C() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(["another-proc", testProcName]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap2, cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),(originalValue.toNumber() + 1), `new value should be ${originalValue.toNumber()+1}`);
            })
            it('C() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([procName+"abc"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),originalValue.toNumber(), `new value should still be ${originalValue.toNumber()}`);
            })
        })
        describe('E() - with data (function selector and arguments) and return', function () {
            const testProcName = "Adder";
            const testContract = Valid.Adder;
            const testBytecode = Valid.Adder.bytecode;
            const functionSpec = "E()";
            it('E() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap2, cap1]));

                const newValue = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);

                assert.equal(newValue.toNumber(),8, `new value should be 8`);
            })
            it('E() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");
            })
            it('E() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");
            })
            it('E() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(["another-proc", testProcName]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap2, cap1]));

                const newValue = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(newValue.toNumber(),8, `new value should be 8`);
            })
            it('E() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([procName+"abc"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                const tx2 = await kernel.registerAnyProcedure(testProcName, deployedTestContract.address, beakerlib.Cap.toInput([cap1]));

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");
            })
        })
        describe('F() - successive calls single depth', function () {
            const testProcName = "Adder";
            const testBytecode = Valid.Adder.bytecode;
            const functionSpec = "F()";
            it('F() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedAdderContract = await testutils.deployedTrimmed(Valid.Adder);
                const deployedSysCallTestContract = await testutils.deployedTrimmed(Valid.SysCallTest);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the first called procedure, which doesn't really do anything
                await kernel.registerProcedure("Adder", deployedAdderContract.address, beakerlib.Cap.toInput([]));
                // // This is the second called procedure, which requires capabilities
                await kernel.registerProcedure("SysCallTest", deployedSysCallTestContract.address, beakerlib.Cap.toInput([cap2, cap1]));

                const newValue = await kernel.executeProcedure.call(procName, functionSpec, "", 32);
                // Execute
                const tx = await kernel.executeProcedure(procName, functionSpec, "", 32);
                // console.log(tx);

                // console.log(tx.receipt.logs)
                // for (const log of tx.receipt.logs) {
                //     if (log.topics.length > 0) {
                //         console.log(`Log: ${web3.toAscii(log.topics[0])} - ${log.data} - ${web3.toAscii(log.data)}`);
                //     } else {
                //         console.log(`Log: ${log.topics[0]} - ${web3.toAscii(log.data)} - ${log.data}`);
                //     }
                // }
                // console.log(web3.toHex(newValue))
                assert.equal(newValue.toNumber(),8, `new value should be 8`);
                const newValue2 =  await kernel.testGetter.call();
                assert.equal(newValue2.toNumber(),4, "new value should be 4");
            })
        })
        describe('G() - deeper stacks', function () {
            const testProcName = "FirstNestedCall";
            const testBytecode = Valid.FirstNestedCall.bytecode;
            const functionSpec = "G()";
            it('G() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedAdderContract = await testutils.deployedTrimmed(Valid.Adder);
                const deployedFirstNestedContract = await testutils.deployedTrimmed(Valid.FirstNestedCall);
                const deployedSecondNestedContract = await testutils.deployedTrimmed(Valid.SecondNestedCall);
                const deployedThirdNestedContract = await testutils.deployedTrimmed(Valid.ThirdNestedCall);
                const deployedFourthNestedContract = await testutils.deployedTrimmed(Valid.FourthNestedCall);
                const deployedFifthNestedContract = await testutils.deployedTrimmed(Valid.FifthNestedCall);
                const deployedSixthNestedContract = await testutils.deployedTrimmed(Valid.SixthNestedCall);
                // This is the procedure that will do the calling
                const tx1 = await kernel.registerProcedure(procName, deployedContract.address, capArray);
                // This is the called procedure
                await kernel.registerProcedure("Adder", deployedAdderContract.address, beakerlib.Cap.toInput([]));
                await kernel.registerProcedure("FirstNestedCall",  deployedFirstNestedContract.address,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8001,0), new beakerlib.CallCap()]));
                await kernel.registerProcedure("SecondNestedCall", deployedSecondNestedContract.address, beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8002,0), new beakerlib.CallCap()]));
                await kernel.registerProcedure("ThirdNestedCall",  deployedThirdNestedContract.address,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8003,0), new beakerlib.CallCap()]));
                await kernel.registerProcedure("FourthNestedCall", deployedFourthNestedContract.address, beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8004,0), new beakerlib.CallCap()]));
                await kernel.registerProcedure("FifthNestedCall",  deployedFifthNestedContract.address,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8005,0), new beakerlib.CallCap()]));
                await kernel.registerProcedure("SixthNestedCall",  deployedSixthNestedContract.address,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8006,0), new beakerlib.CallCap()]));

                await kernel.executeProcedure("FirstNestedCall", "G()", "", 32);

                const firstVal = await kernel.anyTestGetter(0x8001);
                assert.equal(firstVal.toNumber(),75, `new value should be 75`);

                const secondVal = await kernel.anyTestGetter(0x8002);
                assert.equal(secondVal.toNumber(),76, `new value should be 76`);

                const thirdVal = await kernel.anyTestGetter(0x8003);
                assert.equal(thirdVal.toNumber(),77, `new value should be 77`);

                const fourthVal = await kernel.anyTestGetter(0x8004);
                assert.equal(fourthVal.toNumber(),78, `new value should be 78`);

                const fifthVal = await kernel.anyTestGetter(0x8005);
                assert.equal(fifthVal.toNumber(),79, `new value should be 79`);

                const sixthVal = await kernel.anyTestGetter(0x8006);
                assert.equal(sixthVal.toNumber(),80, `new value should be 80`);
            })
        })
    })
})