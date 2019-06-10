const Web3 = require('web3')
const assert = require('assert')

import {newKernelInstance, web3, createAccount, KernelInstance, deployContract } from '../utils'


describe('Write Syscall', function () {
    describe('#getNum', function () {
        it('should return correct value', async function () {
            const accounts = await web3.eth.personal.getAccounts()

            let newProc = await deployContract("writer_test", "TestWriterInterface");
            let kernel = await newKernelInstance("init", newProc.address);

            let kernel_asWriter = newProc.clone();
            kernel_asWriter.address = kernel.contract.address;

            let raw_data = newProc.methods.getNum().encodeABI()

            let result = await web3.eth.call({
                from: accounts[0],
                data: raw_data
            })

            assert.equal(result, 1);
        })
    })
})

