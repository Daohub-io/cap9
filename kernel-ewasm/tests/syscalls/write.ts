const Web3 = require('web3')
const assert = require('assert')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract } from '../utils'


describe('Write Syscall', function () {
    this.timeout(10_000);
    describe('#getNum', function () {
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
            const key = 0;

            // Retrieve the original value.
            const original_value = await newProc.methods.getNum(key).call();
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
            const key = 0;
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Retrieve the original value.
            const original_value = await newProc.methods.getNum(key).call();
            assert.strictEqual(original_value.toNumber(), 0, "The original value should be 0");

            // Write a new value (1) into the storage at 'key'
            await newProc.methods.writeNumDirect(key, 1).send();

            // Retrive the value again and ensure that it has changed.
            const new_value = await newProc.methods.getNum(key).call();
            assert.strictEqual(new_value.toNumber(), 1, "The new value should be 1");
        })
        it.skip('should modify the value via the kernel', async function () {
            const accounts = await web3.eth.personal.getAccounts()

            let newProc = await deployContract("writer_test", "TestWriterInterface");
            let kernel = await newKernelInstance("init", newProc.address);

            // Here we make a copy of the "writer_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the writer contract directly to the kernel.
            let kernel_asWriter = newProc.clone();
            kernel_asWriter.address = kernel.contract.address;


            const kernelAddress = kernel.contract.options.address;
            const code_size = await kernel.contract.methods.get_code_size(newProc.address).call();
            const code_hex = await kernel.contract.methods.code_copy(newProc.address).call();
            const code = Buffer.from(web3.utils.hexToBytes(code_hex));
            // console.log(code)
            // fs.writeFile("t.wasm", code, "binary", ()=>{});
            // assert.strictEqual(code_size, code.length, "The code length should be as given by EXTCODESIZE");


            // The writer_test procedure is now set as the entry procedure. In
            // order to execute this procedure, we first have to put the kernel
            // into "entry procedure mode".
            const toggle1 = await kernel.contract.methods.get_mode().call();
            assert.strictEqual(toggle1, 0, "The kernel should be in test mode (0)");
            await kernel.contract.methods.toggle_syscall().send();
            // Once we have toggled entry procedure on, we have no way to switch
            // back.

            // This is the key that we will be modifying in storage.
            const key = 0;
            // This is the index of the capability (in the procedures capability
            // list) that we will be using to perform the writes.
            const cap_index = 0;

            // Retrieve the original value.
            const original_value = await newProc.methods.getNum(key).call();
            assert.strictEqual(original_value.toNumber(), 0, "The original value should be 0");

            // Write a new value (1) into the storage at 'key' using the cap at
            // 'cap_index'
            // await newProc.methods.writeNum(cap_index, key, 1).send();

            // Retrive the value again and ensure that it has changed.
            // const new_value = await newProc.methods.getNum(key).call();
            // assert.strictEqual(new_value.toNumber(), 1, "The new value should be 1");
        })
    })
})
