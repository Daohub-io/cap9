const Web3 = require('web3')
const assert = require('assert')

import {newKernelInstance, web3, createAccount, KernelInstance, newTestProcedure } from '../integration/utils'


describe('Write Syscall', function () {
    describe('#getNum', function () {
        it('should return correct value', async function () {

            let newProc = await newTestProcedure("writer_test", "TestWriterInterface");
            let kernel = await newKernelInstance("init", newProc.address);

            let kernel_asWriter = newProc.clone();
            kernel_asWriter.address = kernel.contract.address;

            let result = await kernel_asWriter.methods.getNum().call();

            assert.equal(result, 1);
        })
    })
})

