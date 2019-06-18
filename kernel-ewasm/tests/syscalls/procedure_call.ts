const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, CallCap, NewCap } from '../utils'
import { notEqual } from 'assert';


describe('Procedure Call Syscall', function () {
    this.timeout(40_000);
    describe('#callProc', function () {
        it('should return the testNum without a call', async function () {
            const caps = [new NewCap(0, new CallCap(0, "init"))];

            let newProc = await deployContract("caller_test", "TestCallerInterface");
            let kernel = await newKernelInstance("init", newProc.address, caps);

            // Here we make a copy of the "caller_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the caller contract directly to the kernel.
            let kernel_asCaller = newProc.clone();
            kernel_asCaller.address = kernel.contract.address;

            // The caller_test procedure is now set as the entry procedure. In
            // order to execute this procedure, we first have to put the kernel
            // into "entry procedure mode".
            const toggle1 = await kernel.contract.methods.get_mode().call();
            assert.strictEqual(toggle1, 0, "The kernel should be in test mode (0)");
            await kernel.contract.methods.toggle_syscall().send();
            // Once we have toggled entry procedure on, we have no way to switch
            // back.

            // Retrieve the test value.
            const test_value = await kernel_asCaller.methods.testNum().call();
            assert.strictEqual(test_value.toNumber(), 76, "The test value should be 76");
        })
        it('should call itself, with correct cap', async function () {
            const caps = [new NewCap(0, new CallCap(0, "init"))];

            let newProc = await deployContract("caller_test", "TestCallerInterface");
            let kernel = await newKernelInstance("init", newProc.address, caps);

            // Here we make a copy of the "caller_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the writer contract directly to the kernel.
            let kernel_asCaller = newProc.clone();
            kernel_asCaller.address = kernel.contract.address;

            // The caller_test procedure is now set as the entry procedure. In
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

            // Write a new value (1) into the storage at 'key' using the cap at
            // 'cap_index'
            const payload = "0xae28f1ed";
            const message = kernel_asCaller.methods.callProc(cap_index, key, payload).encodeABI();
            const return_value = await web3.eth.call({to:kernel.contract.address, data: message});
            assert.strictEqual(normalize(return_value), normalize(76), `The new value should be ${76}`);
        })
        it('should fail to call itself, with incorrect cap', async function () {
            const cap_key = "abcde";
            const caps = [new NewCap(0, new CallCap(5, cap_key))];

            let newProc = await deployContract("caller_test", "TestCallerInterface");
            let kernel = await newKernelInstance("init", newProc.address, caps);

            // Here we make a copy of the "caller_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the writer contract directly to the kernel.
            let kernel_asCaller = newProc.clone();
            kernel_asCaller.address = kernel.contract.address;

            // The caller_test procedure is now set as the entry procedure. In
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
            const call_cap = await kernel_asCaller.methods.getCap(0x3,0).call();
            // assert.strictEqual(normalize(call_cap[0]), normalize(192), "The prefix size of call cap should be 192");
            // A little bit of padding is added here just for the purposes of a
            // quick test.
            // assert.strictEqual(web3.utils.toHex(call_cap[1]).padEnd(66,'\0'), web3.utils.toHex(web3.utils.fromAscii(cap_key)).padEnd(66,'\0'), `The base key of the write cap should be ${cap_key}`);


            // Write a new value (1) into the storage at 'key' using the cap at
            // 'cap_index'
            const payload = "0xae28f1ed";
            const message = kernel_asCaller.methods.callProc(cap_index, key, payload).encodeABI();
            let success;
            try {
                const return_value = await web3.eth.call({to:kernel.contract.address, data: message})
                success = true;
                console.log(return_value)
            } catch (e) {
                success = false;
            }
            assert(!success, "Call should not succeed");
        })
    })
})
