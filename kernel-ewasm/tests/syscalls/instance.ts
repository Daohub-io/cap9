const Web3 = require('web3')
const assert = require('assert')

import {newKernelInstance, web3, createAccount, KernelInstance, newTestContract } from '../utils'

describe('Kernel', function () {
    describe('#constructor', function () {
        it('should have correct Initial Entry Procedure', async function () {
            let kernel = await newKernelInstance("init", "0xc1912fee45d61c87cc5ea59dae31190fffff232d");

            // Check entryProcedure
            const entryProcedureKey = await kernel.getEntryProcedure()
            assert.strictEqual(entryProcedureKey, "init")

            // Check entryProcedure
            const currentProcedureKey = await kernel.getCurrentProcedure()
            assert.strictEqual(currentProcedureKey, "")
        })
    })

    describe('entry_proc', function() {
        it('should forward call to entry procedure', async function() {
            let newProc = await newTestContract("entry_test", "TestWriterInterface");
            let kernel = await newKernelInstance("init", newProc.address);

            let kernel_asWriter = newProc.clone();
            kernel_asWriter.address = kernel.contract.address;

            let result = await kernel_asWriter.methods.getNum().call();

            assert.equal(result, 1);
        })
    })

})

