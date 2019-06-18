const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, CallCap, NewCap } from '../utils'
import { notEqual } from 'assert';


describe.skip('Register Procedure Syscall', function () {
    this.timeout(40_000);
    describe('#callProc', function () {
        it('should return the testNum without a call', async function () {
            const caps = [new NewCap(0, new CallCap(0, "init"))];

            let newProc = await deployContract("register_test", "TestRegisterInterface");
            let kernel = await newKernelInstance("init", newProc.address, caps);

            // Here we make a copy of the "register_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the register contract directly to the kernel.
            let kernel_asRegister = newProc.clone();
            kernel_asRegister.address = kernel.contract.address;

            // The register_test procedure is now set as the entry procedure. In
            // order to execute this procedure, we first have to put the kernel
            // into "entry procedure mode".
            const toggle1 = await kernel.contract.methods.get_mode().call();
            assert.strictEqual(toggle1, 0, "The kernel should be in test mode (0)");
            await kernel.contract.methods.toggle_syscall().send();
            // Once we have toggled entry procedure on, we have no way to switch
            // back.

            // Retrieve the test value.
            const test_value = await kernel_asRegister.methods.testNum().call();
            assert.strictEqual(test_value.toNumber(), 76, "The test value should be 76");
        })
        it('should call itself, with correct cap', async function () {
            const caps = [new NewCap(0, new CallCap(0, "init"))];

            let newProc = await deployContract("register_test", "TestRegisterInterface");
            let kernel = await newKernelInstance("init", newProc.address, caps);

            // Here we make a copy of the "register_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the writer contract directly to the kernel.
            let kernel_asRegister = newProc.clone();
            kernel_asRegister.address = kernel.contract.address;

            // The register_test procedure is now set as the entry procedure. In
            // order to execute this procedure, we first have to put the kernel
            // into "entry procedure mode".
            const toggle1 = await kernel.contract.methods.get_mode().call();
            assert.strictEqual(toggle1, 0, "The kernel should be in test mode (0)");
            await kernel.contract.methods.toggle_syscall().send();
            // Once we have toggled entry procedure on, we have no way to switch
            // back.

            // This is the key that we will be modifying in storage.
            const key = "0x" + web3.utils.fromAscii("init",24).slice(2).padStart(64,"0");
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Here we prepare a message to call the "testNum()" method, but
            // rather than send it we just keep it as an encoded message (called
            // payload).
            const payload = kernel_asRegister.methods.testNum().encodeABI();
            // We then send that message via a call procedure syscall.
            const message = kernel_asRegister.methods.callProc(cap_index, key, payload).encodeABI();
            const return_value = await web3.eth.call({to:kernel.contract.address, data: message});
            assert.strictEqual(normalize(return_value), normalize(76), `The new value should be ${76}`);
        })
        it('should fail to call itself, with incorrect cap', async function () {
            const cap_key = "abcde";
            const prefix = 5;
            const caps = [new NewCap(0, new CallCap(prefix, cap_key))];

            let newProc = await deployContract("register_test", "TestRegisterInterface");
            let kernel = await newKernelInstance("init", newProc.address, caps);

            // Here we make a copy of the "register_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the writer contract directly to the kernel.
            let kernel_asRegister = newProc.clone();
            kernel_asRegister.address = kernel.contract.address;

            // The register_test procedure is now set as the entry procedure. In
            // order to execute this procedure, we first have to put the kernel
            // into "entry procedure mode".
            const toggle1 = await kernel.contract.methods.get_mode().call();
            assert.strictEqual(toggle1, 0, "The kernel should be in test mode (0)");
            await kernel.contract.methods.toggle_syscall().send();
            // Once we have toggled entry procedure on, we have no way to switch
            // back.

            // This is the key that we will be modifying in storage.
            const key = "0x" + web3.utils.fromAscii("init",24).slice(2).padStart(64,"0");
            const value = "0xae28f1ed";
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Check that we have the right cap at index 0
            const call_cap = await kernel_asRegister.methods.getCap(0x3,0).call();
            assert.strictEqual(normalize(call_cap[0]), normalize(prefix), "The prefix size of call cap should be 192");
            // A little bit of padding is added here just for the purposes of a
            // quick test.
            assert.strictEqual(web3.utils.toHex(call_cap[1]).padEnd(66,'0').slice(2).padStart(66,'0'), web3.utils.toHex(web3.utils.fromAscii(cap_key)).padEnd(66,'0').slice(2).padStart(66,'0'), `The base key of the write cap should be ${cap_key}`);

            // Here we prepare a message to call the "testNum()" method, but
            // rather than send it we just keep it as an encoded message (called
            // payload).
            const payload = kernel_asRegister.methods.testNum().encodeABI();
            // We then send that message via a call procedure syscall.
            const message = kernel_asRegister.methods.callProc(cap_index, key, payload).encodeABI();
            let success;
            try {
                const return_value = await web3.eth.call({to:kernel.contract.address, data: message})
                success = true;
            } catch (e) {
                success = false;
            }
            assert(!success, "Call should not succeed");
        })
    })
})
