const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, LogCap, NewCap } from '../utils'
import { notEqual } from 'assert';


describe('Log Syscall', function () {
    this.timeout(40_000);
    describe('No topics', function () {
        it('should log a value with correct cap', async function () {
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
            const message = kernel_asLogger.methods.log(cap_index, [], value).encodeABI();
            const return_value = await web3.eth.sendTransaction({to:kernel.contract.address, data: message})
            assert.strictEqual(normalize(return_value.logs[0].data), normalize(value), "The correct value should be logged");
        })
        it('should fail when no cap is given', async function () {
            const caps = [];

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

            const value = "0xabcdabcd";
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Write a new value (1) into the storage at 'key' using the cap at
            // 'cap_index'
            const message = kernel_asLogger.methods.log(cap_index, [], value).encodeABI();
            let return_value;
            let success;
            try {
                return_value = await web3.eth.sendTransaction({to:kernel.contract.address, data: message});
                success = true;
                console.log(return_value)
            } catch (e) {
                success = false;
            }
            assert(!success, "Call should not succeed");
            // assert.strictEqual(return_value.status, false, "The log syscall should fail");
            // assert.strictEqual(return_value.logs.length, 0, "Nothing should be logged");
        })
        it('should fail when cap requires more topics', async function () {
            const caps = [new NewCap(0, new LogCap(["abcde"]))];

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

            const value = "0xabcdabcd";
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Write a new value (1) into the storage at 'key' using the cap at
            // 'cap_index'
            const message = kernel_asLogger.methods.log(cap_index, [], value).encodeABI();
            let return_value;
            let success;
            try {
                return_value = await web3.eth.sendTransaction({to:kernel.contract.address, data: message});
                success = true;
                console.log(return_value)
            } catch (e) {
                success = false;
            }
            assert(!success, "Call should not succeed");
            // assert.strictEqual(return_value.status, false, "The log syscall should fail");
            // assert.strictEqual(return_value.logs.length, 0, "Nothing should be logged");
        })
    })
    describe('Single topic', function () {
        it('should log a value with correct cap', async function () {
            const caps = [new NewCap(0, new LogCap(["abcd"]))];

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

            const log_cap = await kernel_asLogger.methods.getCap(0x8,0).call();

            // This is the key that we will be modifying in storage.
            // const key = web3.utils.fromAscii("init");
            const value = "0xabcdabcd";
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Write a new value (1) into the storage at 'key' using the cap at
            // 'cap_index'
            const message = kernel_asLogger.methods.log(cap_index, [web3.utils.fromAscii("abcd",32)], value).encodeABI();
            // console.log(message)
            const return_value = await web3.eth.sendTransaction({to:kernel.contract.address, data: message})
            assert.strictEqual(normalize(return_value.logs[0].data), normalize(value), "The correct value should be logged");
        })
        it('should fail to log with incorrect cap', async function () {
            const caps = [new NewCap(0, new LogCap(["xyz"]))];

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

            const value = "0xabcdabcd";
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Write a new value (1) into the storage at 'key' using the cap at
            // 'cap_index'
            const message = kernel_asLogger.methods.log(cap_index, [web3.utils.fromAscii("abcd")], value).encodeABI();
            let return_value;
            let success;
            try {
                return_value = await web3.eth.sendTransaction({to:kernel.contract.address, data: message});
                success = true;
                console.log(return_value)
            } catch (e) {
                success = false;
            }
            assert(!success, "Call should not succeed");
            // assert.strictEqual(return_value.status, false, "The log syscall should fail");
            // assert.strictEqual(return_value.logs.length, 0, "Nothing should be logged");
        })
    })
})
