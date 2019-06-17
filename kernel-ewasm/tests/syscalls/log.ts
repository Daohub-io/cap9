const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, LogCap, NewCap } from '../utils'
import { notEqual } from 'assert';


describe.only('Log Syscall', function () {
    this.timeout(40_000);
    describe('#log', function () {
        it.only('should return the testNum', async function () {
            const caps = [new NewCap(0, new LogCap([]))];

            let newProc = await deployContract("logger_test", "TestLoggerInterface");
            let kernel = await newKernelInstance("init", newProc.address, caps);

            // Here we make a copy of the "logger_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the logger contract directly to the kernel.
            let kernel_asLogger = newProc.clone();
            kernel_asLogger.address = kernel.contract.address;

            // The logger_test procedure is now set as the entry procedure. In
            // order to execute this procedure, we first have to put the kernel
            // into "entry procedure mode".
            const toggle1 = await kernel.contract.methods.get_mode().call();
            assert.strictEqual(toggle1, 0, "The kernel should be in test mode (0)");
            await kernel.contract.methods.toggle_syscall().send();
            // Once we have toggled entry procedure on, we have no way to switch
            // back.

            // Retrieve the test value.
            const test_value = await kernel_asLogger.methods.testNum().call();
            assert.strictEqual(test_value.toNumber(), 76, "The test value should be 76");
        })
        it.only('should log a value with correct cap', async function () {
            const caps = [new NewCap(0, new LogCap([]))];

            let newProc = await deployContract("logger_test", "TestLoggerInterface");
            let kernel = await newKernelInstance("init", newProc.address, caps);

            // Here we make a copy of the "logger_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the writer contract directly to the kernel.
            let kernel_asLogger = newProc.clone();
            kernel_asLogger.address = kernel.contract.address;

            // The logger_test procedure is now set as the entry procedure. In
            // order to execute this procedure, we first have to put the kernel
            // into "entry procedure mode".
            const toggle1 = await kernel.contract.methods.get_mode().call();
            assert.strictEqual(toggle1, 0, "The kernel should be in test mode (0)");
            await kernel.contract.methods.toggle_syscall().send();
            // Once we have toggled entry procedure on, we have no way to switch
            // back.

            // This is the key that we will be modifying in storage.
            // const key = web3.utils.fromAscii("init");
            const value = "0xabcdabcd";
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Write a new value (1) into the storage at 'key' using the cap at
            // 'cap_index'
            const message = kernel_asLogger.methods.log(cap_index, value).encodeABI();
            const return_value = await web3.eth.call({to:kernel.contract.address, data: message})
            console.log(return_value)
            // assert.strictEqual(normalize(return_value), normalize(76), `The new value should be ${76}`);
        })
        it.skip('should fail to call itself, with incorrect cap', async function () {
            const caps = [new NewCap(0, new LogCap([]))];

            let newProc = await deployContract("logger_test", "TestLoggerInterface");
            let kernel = await newKernelInstance("init", newProc.address, caps);

            // Here we make a copy of the "logger_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the writer contract directly to the kernel.
            let kernel_asLogger = newProc.clone();
            kernel_asLogger.address = kernel.contract.address;

            // The logger_test procedure is now set as the entry procedure. In
            // order to execute this procedure, we first have to put the kernel
            // into "entry procedure mode".
            const toggle1 = await kernel.contract.methods.get_mode().call();
            assert.strictEqual(toggle1, 0, "The kernel should be in test mode (0)");
            await kernel.contract.methods.toggle_syscall().send();
            // Once we have toggled entry procedure on, we have no way to switch
            // back.

            // This is the key that we will be modifying in storage.
            const key = web3.utils.fromAscii("init");
            const value = "0xae28f1ed";
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Check that we have the right cap at index 0
            const call_cap = await kernel_asLogger.methods.getCap(0x3,0).call();
            console.log(call_cap);
            console.log(web3.utils.toHex(call_cap[0]),web3.utils.toHex(call_cap[1]));
            assert.strictEqual(normalize(call_cap[0]), normalize(192), "The prefix size of call cap should be 192");
            assert.strictEqual(normalize(call_cap[1]), normalize(key), `The base key of the write cap should be ${key}`);


            // Write a new value (1) into the storage at 'key' using the cap at
            // 'cap_index'
            // const message = kernel_asLogger.methods.callProc(cap_index, key).encodeABI();
            // let success;
            // try {
            //     const return_value = await web3.eth.call({to:kernel.contract.address, data: message})
            //     success = true;
            //     console.log(return_value)
            // } catch (e) {
            //     success = false;
            // }
            // assert(!success, "Call should not succeed");
        })
    })
})
