const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, WriteCap, NewCap } from '../utils'
import { notEqual } from 'assert';


describe('Write Syscall', function () {
    this.timeout(40_000);
    describe('#get/writeNum', function () {
        it('should return the initial value', async function () {
            const accounts = await web3.eth.personal.getAccounts()

            let newProc = await deployContract("writer_test", "TestWriterInterface");
            let kernel = await newKernelInstance("init", newProc.address);

            // Here we make a copy of the "writer_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the writer contract directly to the kernel.
            let kernel_asWriter = newProc.clone();
            kernel_asWriter.address = kernel.contract.address;

            // The writer_test procedure is now set as the entry procedure. In
            // order to execute this procedure, we first have to put the kernel
            // into "entry procedure mode".
            const toggle1 = await kernel.contract.methods.get_mode().call();
            assert.strictEqual(toggle1, 0, "The kernel should be in test mode (0)");
            await kernel.contract.methods.toggle_syscall().send();
            // Once we have toggled entry procedure on, we have no way to switch
            // back.

            // This is the key that we will be modifying in storage.
            const key = "0xdeadbeef";

            // Retrieve the original value.
            const original_value = await kernel_asWriter.methods.getNum(key).call();
            assert.strictEqual(original_value.toNumber(), 0, "The original value should be 0");
        })
        it('should modify the value directly', async function () {
            const accounts = await web3.eth.personal.getAccounts()

            let newProc = await deployContract("writer_test", "TestWriterInterface");
            let kernel = await newKernelInstance("init", newProc.address);

            // Here we make a copy of the "writer_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the writer contract directly to the kernel.
            let kernel_asWriter = newProc.clone();
            kernel_asWriter.address = kernel.contract.address;

            // The writer_test procedure is now set as the entry procedure. In
            // order to execute this procedure, we first have to put the kernel
            // into "entry procedure mode".
            const toggle1 = await kernel.contract.methods.get_mode().call();
            assert.strictEqual(toggle1, 0, "The kernel should be in test mode (0)");
            await kernel.contract.methods.toggle_syscall().send();
            // Once we have toggled entry procedure on, we have no way to switch
            // back.

            // This is the key that we will be modifying in storage.
            const key = "0xdeadbeef";
            const value = "0xfee";
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Retrieve the original value.
            const original_value = await kernel_asWriter.methods.getNum(key).call();
            assert.strictEqual(original_value.toNumber(), 0, "The original value should be 0");

            // Write a new value (1) into the storage at 'key' using the cap at
            // 'cap_index'
            await kernel_asWriter.methods.writeNumDirect(key, value).send();
            // await kernel_asWriter.methods.writeNum(cap_index, key, 1).call();

            // Retrive the value again and ensure that it has changed.
            const new_value = await kernel_asWriter.methods.getNum(key).call();
            assert.strictEqual(normalize(new_value), normalize(value), `The new value should be ${value}`);
        })
        it('should modify the value via the kernel, with correct cap', async function () {
            const caps = [new NewCap(0, new WriteCap(0xdeadbeef, 2))];

            let newProc = await deployContract("writer_test", "TestWriterInterface");
            let kernel = await newKernelInstance("init", newProc.address, caps);

            // Here we make a copy of the "writer_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the writer contract directly to the kernel.
            let kernel_asWriter = newProc.clone();
            kernel_asWriter.address = kernel.contract.address;

            // The writer_test procedure is now set as the entry procedure. In
            // order to execute this procedure, we first have to put the kernel
            // into "entry procedure mode".
            const toggle1 = await kernel.contract.methods.get_mode().call();
            assert.strictEqual(toggle1, 0, "The kernel should be in test mode (0)");
            await kernel.contract.methods.toggle_syscall().send();
            // Once we have toggled entry procedure on, we have no way to switch
            // back.

            // This is the key that we will be modifying in storage.
            const key = "0xdeadbeef";
            const value = "0xfee";
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Retrieve the original value.
            const original_value = await kernel_asWriter.methods.getNum(key).call();
            assert.strictEqual(original_value.toNumber(), 0, "The original value should be 0");

            // Check that we have the right cap at index 0
            const write_cap = await kernel_asWriter.methods.getCap(0x7,0).call();
            assert.strictEqual(normalize(write_cap[0]), normalize(key), "The base of the write cap should match the key");
            assert.strictEqual(normalize(write_cap[1]), normalize(2), "The additional length of the write cap should be 2");

            // Write a new value (1) into the storage at 'key' using the cap at
            // 'cap_index'
            await kernel_asWriter.methods.writeNum(cap_index, key, value).send();

            // Retrive the value again and ensure that it has changed.
            const new_value = await kernel_asWriter.methods.getNum(key).call();
            assert.strictEqual(normalize(new_value), normalize(value), `The new value should be ${value}`);
        })
        it('should fail to modify the value via the kernel with the wrong cap', async function () {
            const caps = [new NewCap(0, new WriteCap(0x8000, 2))];

            let newProc = await deployContract("writer_test", "TestWriterInterface");
            let kernel = await newKernelInstance("init", newProc.address, caps);

            // Here we make a copy of the "writer_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the writer contract directly to the kernel.
            let kernel_asWriter = newProc.clone();
            kernel_asWriter.address = kernel.contract.address;

            // The writer_test procedure is now set as the entry procedure. In
            // order to execute this procedure, we first have to put the kernel
            // into "entry procedure mode".
            const toggle1 = await kernel.contract.methods.get_mode().call();
            assert.strictEqual(toggle1, 0, "The kernel should be in test mode (0)");
            await kernel.contract.methods.toggle_syscall().send();
            // Once we have toggled entry procedure on, we have no way to switch
            // back.

            // This is the key that we will be modifying in storage.
            const key = "0xdeadbeef";
            const value = "0xfee";
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Retrieve the original value.
            const original_value = await kernel_asWriter.methods.getNum(key).call();
            assert.strictEqual(original_value.toNumber(), 0, "The original value should be 0");

            // Check that we have the right cap at index 0
            const write_cap = await kernel_asWriter.methods.getCap(0x7,0).call();
            assert.strictEqual(normalize(write_cap[0]), normalize(0x8000), "The base of the write cap should be 0x8000");
            assert.strictEqual(normalize(write_cap[1]), normalize(2), "The additional length of the write cap should be 2");

            // Write a new value (1) into the storage at 'key' using the cap at
            // 'cap_index'. This should fail.
            try {
                await kernel_asWriter.methods.writeNum(cap_index, key, value).send();
                assert(false, "This should fail");
            } catch (e) {

            }

            // Retrive the value again and ensure that it has changed.
            const new_value = await kernel_asWriter.methods.getNum(key).call();
            assert.strictEqual(normalize(new_value), normalize(0), `The new value should still be ${0}`);
        })
    })
})
