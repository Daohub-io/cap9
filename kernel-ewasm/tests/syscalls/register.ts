const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, CallCap, NewCap, RegisterCap, WriteCap } from '../utils'
import { notEqual } from 'assert';


describe('Register Procedure Syscall', function () {
    this.timeout(40_000);
    describe('#regProc', function () {
        it('should return the testNum', async function () {
            const caps = [new NewCap(0, new RegisterCap(0, "write"))];

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
        it('should register write procedure, with correct cap', async function () {
            const caps = [new NewCap(0, new RegisterCap(5, "write"))];

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

            // This is the key of the procedure that we will be registering.
            const key = "0x" + web3.utils.fromAscii("write", 24).slice(2).padStart(64, "0");
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Deploy the contract for the procedure that we will register.
            let writeProc = await deployContract("writer_test", "TestWriterInterface");

            // This is the address of the new procedure that we wish to register.

            const procList1 = await kernel_asRegister.methods.listProcs().call().then(x=>x.map(normalize));
            // We then send that message via a call procedure syscall.
            const message = kernel_asRegister.methods.regProc(cap_index, key, writeProc.address, []).encodeABI();
            const return_value = await web3.eth.sendTransaction({ to: kernel.contract.address, data: message });
            const procList2 = await kernel_asRegister.methods.listProcs().call().then(x=>x.map(normalize));
            assert.strictEqual(procList2.length, procList1.length + 1, "The number of procedures should have increased by 1");
            assert(procList2.includes(normalize(web3.utils.fromAscii("write",24))), "The new procedure key should be included in the table");
        })
        it('should fail to register, with no cap', async function () {
            const caps = [];

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

            // This is the key of the procedure that we will be registering.
            const key = "0x" + web3.utils.fromAscii("write", 24).slice(2).padStart(64, "0");
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Deploy the contract for the procedure that we will register.
            let writeProc = await deployContract("writer_test", "TestWriterInterface");

            // This is the address of the new procedure that we wish to register.

            const procList1 = await kernel_asRegister.methods.listProcs().call().then(x=>x.map(normalize));
            // We then send that message via a call procedure syscall.
            const message = kernel_asRegister.methods.regProc(cap_index, key, writeProc.address, []).encodeABI();
            let success;
            try {
                const return_value = await web3.eth.sendTransaction({ to: kernel.contract.address, data: message });
                success = true;
            } catch (e) {
                success = false;
            }
            assert(!success, "Call should not succeed");
            const procList2 = await kernel_asRegister.methods.listProcs().call().then(x=>x.map(normalize));
            assert.strictEqual(procList2.length, procList1.length, "The number of procedures should not have increased");
            assert(!procList2.includes(normalize(web3.utils.fromAscii("write",24))), "The new procedure key should not be included in the table");

        })
        it.skip('should fail to register, with incorrect cap', async function () {
            const caps = [new NewCap(0, new RegisterCap(5, "read"))];

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

            // This is the key of the procedure that we will be registering.
            const key = "0x" + web3.utils.fromAscii("write", 24).slice(2).padStart(64, "0");
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Deploy the contract for the procedure that we will register.
            let writeProc = await deployContract("writer_test", "TestWriterInterface");

            // This is the address of the new procedure that we wish to register.

            const procList1 = await kernel_asRegister.methods.listProcs().call().then(x=>x.map(normalize));
            // We then send that message via a call procedure syscall.
            const message = kernel_asRegister.methods.regProc(cap_index, key, writeProc.address, []).encodeABI();
            let success;
            try {
                const return_value = await web3.eth.sendTransaction({ to: kernel.contract.address, data: message });
                success = true;
            } catch (e) {
                success = false;
            }
            assert(!success, "Call should not succeed");
            const procList2 = await kernel_asRegister.methods.listProcs().call().then(x=>x.map(normalize));
            assert.strictEqual(procList2.length, procList1.length, "The number of procedures should not have increased");
            assert(!procList2.includes(normalize(web3.utils.fromAscii("write",24))), "The new procedure key should not be included in the table");

        })
    })
})
