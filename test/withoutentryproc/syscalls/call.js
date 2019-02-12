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
        const procName = web3.utils.utf8ToHex("SysCallTestCall");
        const contract = Valid.SysCallTestCall;
        const bytecode = Valid.SysCallTestCall.bytecode;

        describe('A() - call procedure which needs no caps', function () {
            const testProcName = "TestWrite"
            const testProcNameHex = web3.utils.utf8ToHex(testProcName);
            const testBytecode = TestWrite.bytecode;
            const testContract = TestWrite;
            const functionSpec = "A()";
            it('A() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const addresses = new Map();

                addresses.set(kernel.address, "kernel");
                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                addresses.set(deployedContract.options.address, procName);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                addresses.set(deployedTestContract.options.address, testProcNameHex);

                // This is the procedure that will do the calling
                let tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();           

                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, []).send();
                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                
                const tx3 = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send()
                
                assert.equal(valueX, 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue,356, "new value should be 356");
            })
            it('A() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);

                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, []).send();

                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 222233);

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue, 3, "new value should still be 3");
            })
            it('A() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, []).send();

                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue,3, "new value should still be 3");
            })
            it('A() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(["another-proc", testProcName]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);

                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, []).send();

                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 0);

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue,356, "new value should be 356");
            })
            it('A() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([procName+"abc"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, []).send();

                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 222233);

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue,3, "new value should still be 3");
            })
        })
        describe('B() - without data', function () {
            const testProcNameHex = web3.utils.utf8ToHex("SysCallTest");
            const testContract = Valid.SysCallTest;
            const testBytecode = Valid.SysCallTest.bytecode;
            const functionSpec = "B()";
            it('B() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const addresses = new Map();
                addresses.set(kernel.options.address, "kernel");

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                addresses.set(deployedContract.options.address, procName);
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap2, cap1]));
                addresses.set(deployedTestContract.options.address, testProcNameHex);

                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();

                assert.equal(valueX, 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue,356, "new value should be 356");
            })
            it('B() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);


                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap1])).send();

                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue,3, "new value should still be 3");
            })
            it('B() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap1])).send();

                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue,3, "new value should still be 3");
            })
            it('B() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(["another-proc", testProcNameHex]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap2, cap1])).send();

                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue,356, "new value should be 356");
            })
            it('B() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([procName+"abc"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap1])).send();

                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue,3, "new value should still be 3");
            })
        })
        describe('C() - with data (function selector)', function () {
            const testProcNameHex = web3.utils.utf8ToHex("SysCallTest");
            const testBytecode = Valid.SysCallTest.bytecode;
            const testContract = Valid.SysCallTest;
            const functionSpec = "C()";
            it('C() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap2, cap1])).send();

                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();

                assert.equal(valueX, 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue,(originalValue + 1), `new value should be ${originalValue+1}`);
            })
            it('C() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap1])).send();

                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue,originalValue, `new value should still be ${originalValue}`);
            })
            it('C() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap1])).send();

                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue,originalValue, `new value should still be ${originalValue}`);
            })
            it('C() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(["another-proc", testProcNameHex]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap2, cap1])).send();

                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 0, "should succeed with zero errcode the first time");

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue,(originalValue + 1), `new value should be ${originalValue+1}`);
            })
            it('C() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([procName+"abc"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap1])).send();

                const originalValue =  await kernel.methods.testGetter().call();
                assert.equal(originalValue, 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.methods.testGetter().call();
                assert.equal(newValue,originalValue, `new value should still be ${originalValue}`);
            })
        })
        describe('E() - with data (function selector and arguments) and return', function () {
            const testProcNameHex = web3.utils.utf8ToHex("Adder");
            const testContract = Valid.Adder;
            const testBytecode = Valid.Adder.bytecode;
            const functionSpec = "E()";
            it('E() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap2, cap1])).send();

                const newValue = await kernel.methods.executeProcedure(procName, functionSpec, "", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "", 32).send();

                assert.equal(newValue,8, `new value should be 8`);
            })
            it('E() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap1])).send();

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 222233, "should succeed with zero errcode the first time");
            })
            it('E() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap1])).send();

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 222233, "should succeed with zero errcode the first time");
            })
            it('E() should succeed with a more restricted cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap(["another-proc", testProcNameHex]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap2, cap1])).send();

                const newValue = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(newValue,8, `new value should be 8`);
            })
            it('E() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([procName+"abc"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedTestContract = await testutils.deployedTrimmed(testContract);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                const tx2 = await kernel.methods.registerAnyProcedure(testProcNameHex, deployedTestContract.options.address, beakerlib.Cap.toInput([cap1])).send();

                const valueX = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(valueX, 222233, "should succeed with zero errcode the first time");
            })
        })
        describe('F() - successive calls single depth', function () {
            const testProcNameHex = web3.utils.utf8ToHex("Adder");
            const testBytecode = Valid.Adder.bytecode;
            const functionSpec = "F()";
            it('F() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                const deployedContract = await testutils.deployedTrimmed(contract);
                const deployedAdderContract = await testutils.deployedTrimmed(Valid.Adder);
                const deployedSysCallTestContract = await testutils.deployedTrimmed(Valid.SysCallTest);
                // This is the procedure that will do the calling
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the first called procedure, which doesn't really do anything
                await kernel.methods.registerProcedure(testProcNameHex, deployedAdderContract.address, beakerlib.Cap.toInput([])).send();
                // // This is the second called procedure, which requires capabilities
                await kernel.methods.registerProcedure(web3.utils.utf8ToHex("SysCallTest"), deployedSysCallTestContract.address, beakerlib.Cap.toInput([cap2, cap1])).send();

                const newValue = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).call();
                const tx = await kernel.methods.executeProcedure(procName, functionSpec, "0x", 32).send();
                assert.equal(newValue,8, `new value should be 8`);

                const newValue2 =  await kernel.methods.testGetter().call();
                assert.equal(newValue2,4, "new value should be 4");
            })
        })
        describe('G() - deeper stacks', function () {
            const testProcNameHex = web3.utils.utf8ToHex("FirstNestedCall");
            const testBytecode = Valid.FirstNestedCall.bytecode;
            const functionSpec = "G()";
            it('G() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from x to x+1.
                const kernel = (await Kernel.new()).contract;
                kernel.options.from = Kernel.class_defaults.from;
                kernel.options.gas = 10**6

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
                const tx1 = await kernel.methods.registerProcedure(procName, deployedContract.options.address, capArray).send();
                // This is the called procedure
                await kernel.methods.registerProcedure(testProcNameHex, deployedAdderContract.address, beakerlib.Cap.toInput([])).send();
                await kernel.methods.registerProcedure(web3.utils.utf8ToHex("FirstNestedCall"),  deployedFirstNestedContract.address,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8001,0), new beakerlib.CallCap()])).send();
                await kernel.methods.registerProcedure(web3.utils.utf8ToHex("SecondNestedCall"), deployedSecondNestedContract.address, beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8002,0), new beakerlib.CallCap()])).send();
                await kernel.methods.registerProcedure(web3.utils.utf8ToHex("ThirdNestedCall"),  deployedThirdNestedContract.address,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8003,0), new beakerlib.CallCap()])).send();
                await kernel.methods.registerProcedure(web3.utils.utf8ToHex("FourthNestedCall"), deployedFourthNestedContract.address, beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8004,0), new beakerlib.CallCap()])).send();
                await kernel.methods.registerProcedure(web3.utils.utf8ToHex("FifthNestedCall"),  deployedFifthNestedContract.address,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8005,0), new beakerlib.CallCap()])).send();
                await kernel.methods.registerProcedure(web3.utils.utf8ToHex("SixthNestedCall"),  deployedSixthNestedContract.address,  beakerlib.Cap.toInput([cap2, new beakerlib.WriteCap(0x8006,0), new beakerlib.CallCap()])).send();

                await kernel.methods.executeProcedure(web3.utils.utf8ToHex("FirstNestedCall"), "G()", "0x", 32).send();

                const firstVal = await kernel.methods.anyTestGetter(0x8001).call();
                assert.equal(firstVal,75, `new value should be 75`);

                const secondVal = await kernel.methods.anyTestGetter(0x8002).call();
                assert.equal(secondVal,76, `new value should be 76`);

                const thirdVal = await kernel.methods.anyTestGetter(0x8003).call();
                assert.equal(thirdVal,77, `new value should be 77`);

                const fourthVal = await kernel.methods.anyTestGetter(0x8004).call();
                assert.equal(fourthVal,78, `new value should be 78`);

                const fifthVal = await kernel.methods.anyTestGetter(0x8005).call();
                assert.equal(fifthVal,79, `new value should be 79`);

                const sixthVal = await kernel.methods.anyTestGetter(0x8006).call();
                assert.equal(sixthVal,80, `new value should be 80`);
            })
        })
    })
})