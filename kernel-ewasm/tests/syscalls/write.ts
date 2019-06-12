const Web3 = require('web3')
const assert = require('assert')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract } from '../utils'


describe('Write Syscall', function () {
    this.timeout(40_000);
    describe('#getNum', function () {
        it('should return correct value', async function () {
            const accounts = await web3.eth.personal.getAccounts()

            let newProc = await deployContract("writer_test", "TestWriterInterface");
            let kernel = await newKernelInstance("init", newProc.address);

            // Here we make a copy of the "writer_test" contract interface, but
            // change the address so that it's pointing at the kernel. This
            // means the web3 library will send a message crafted to be read by
            // the writer contract directly to the kernel.
            let kernel_asWriter = newProc.clone();
            kernel_asWriter.address = kernel.contract.address;

            {
                let raw_data = newProc.methods.writeNum(0, 0, 1).encodeABI()
                // This needs to be a "sendTransaction" instead of a "call" in
                // order for it to be able to change state on the chain.
                let result = await web3.eth.sendTransaction({
                    from: accounts[0],
                    data: raw_data
                })
            }

            let raw_data = newProc.methods.getNum(0).encodeABI()
            let result = await web3.eth.call({
                from: accounts[0],
                data: raw_data
            })

            assert.equal(result, 1);
        })
    })
})
