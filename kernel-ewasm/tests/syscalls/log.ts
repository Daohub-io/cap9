const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, LogCap, NewCap } from '../utils'
import { notEqual } from 'assert';


describe('Log Syscall', function () {
    this.timeout(40_000);
    describe('No topics', function () {
        it('should log a value with correct cap', async function () {
            const capTopics = [];
            const logTopics = [];
            const data = "0xabcdabcd";
            const result = true;
            await logCapTest(capTopics, logTopics, data, result);
        })
        it('should fail when no cap is given', async function () {
            const capTopics = null;
            const logTopics = [];
            const data = "0xabcdabcd";
            const result = false;
            await logCapTest(capTopics, logTopics, data, result);
        })
        it('should fail when cap requires more topics', async function () {
            const capTopics = ["abcd"];
            const logTopics = [];
            const data = "0xabcdabcd";
            const result = false;
            await logCapTest(capTopics, logTopics, data, result);
        })
    })
    describe('Single topic', function () {
        it('should log a value with correct cap', async function () {
            const capTopics = ["abcd"];
            const logTopics = ["abcd"];
            const data = "0xabcdabcd";
            const result = true;
            await logCapTest(capTopics, logTopics, data, result);
        })
        it('should fail to log with incorrect cap', async function () {
            const capTopics = ["xyz"];
            const logTopics = ["abcd"];
            const data = "0xabcdabcd";
            const result = false;
            await logCapTest(capTopics, logTopics, data, result);
        })
    })
    describe('Two topics', function () {
        it('should log a value with correct cap', async function () {
            const capTopics = ["abcd","efgh"];
            const logTopics = ["abcd","efgh"];
            const data = "0xabcdabcd";
            const result = true;
            await logCapTest(capTopics, logTopics, data, result);
        })
        it('should fail to log with incorrect cap', async function () {
            const capTopics = ["xyz","abc"];
            const logTopics = ["abcd","efgh"];
            const data = "0xabcdabcd";
            const result = false;
            await logCapTest(capTopics, logTopics, data, result);
        })
    })
})


// We want to test many combinations, so here we define a standard test for log
// cap. Result is a boolean value indicating whether the log should be
// successful or not. If capTopics is null, not cap will be provided. data
// should be a hex string.
async function logCapTest(capTopics, logTopics, data, result: boolean) {
    const caps = capTopics === null ? [] : [new NewCap(0, new LogCap(capTopics))];

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

    // This is the index of the capability (in the procedures capability
    // list) that we will be using to perform the writes.
    const cap_index = 0;

    // Write a new value (1) into the storage at 'key' using the cap at
    // 'cap_index'
    const message = kernel_asLogger.methods.log(cap_index, logTopics.map(x=>web3.utils.fromAscii(x,32)), data).encodeABI();
    let return_value;
    let success;
    try {
        return_value = await web3.eth.sendTransaction({to:kernel.contract.address, data: message});
        success = true;
    } catch (e) {
        success = false;
    }
    if (result) {
        // Check that the correct data is logged.
        assert.strictEqual(normalize(return_value.logs[0].data), normalize(data), "The correct value should be logged");
        // Check that each topic is correctly used
        for (const i in return_value.logs[0].topics) {
            assert.strictEqual(normalize(return_value.logs[0].topics[i]), normalize(web3.utils.fromAscii(logTopics[i],32)), "The correct topics should be used")
        }
    } else {
        assert(!success, "Call should not succeed");
        // assert.strictEqual(return_value.status, false, "The log syscall should fail");
        // assert.strictEqual(return_value.logs.length, 0, "Nothing should be logged");
    }
}
