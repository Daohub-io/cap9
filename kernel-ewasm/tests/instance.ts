const Web3 = require('web3')
const assert = require('assert')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, NewCap, WriteCap, CAP_TYPE, CallCap, LogCap, RegisterCap, DeleteCap, EntryCap, AccCallCap } from './utils'
import { Contract } from 'web3-eth-contract';

describe('Kernel', function () {
    describe('#constructor', function () {
        this.timeout(40_000);
        it('should have correct Initial Entry Procedure', async function () {
            let kernel = await newKernelInstance("init", "0xc1912fee45d61c87cc5ea59dae31190fffff232d");

            // Check entryProcedure
            const entryProcedureKey = await kernel.getEntryProcedure()
            assert.strictEqual(entryProcedureKey, "init")

            // Check currentProcedure
            const currentProcedureKey = await kernel.getCurrentProcedure()
            assert.strictEqual(currentProcedureKey, "")

            // Check all Cap lists
            for (const captype in CAP_TYPE) {
                if (typeof CAP_TYPE[captype] == "number") continue;
                // Check entryProcedure captype Length
                const currentCapLen = await kernel.getProcCapTypeLen("init", captype as any);
                assert.strictEqual(currentCapLen, 0, `There should be 0 of type: ${CAP_TYPE[captype]}`)
            }
        })

        it('should insert capability', async function () {
            let write_cap = new NewCap(0, new WriteCap(0, 100));
            let call_cap = new NewCap(0, new CallCap(0, "init"));
            let log_cap = new NewCap(0, new LogCap(["help"]));
            let reg_cap = new NewCap(0, new RegisterCap(0, "init"));
            let del_cap = new NewCap(0, new DeleteCap(0, "init"));
            let entry_cap = new NewCap(0, new EntryCap());
            let acc_call_cap = new NewCap(0, new AccCallCap(true, true, "0xc1912fee45d61c87cc5ea59dae31190fffff232d"));

            let kernel = await newKernelInstance("init", "0xc1912fee45d61c87cc5ea59dae31190fffff232d", [write_cap, call_cap, log_cap, reg_cap, del_cap, entry_cap, acc_call_cap]);

            // Check all Cap lists
            for (const captype in CAP_TYPE) {
                if (typeof CAP_TYPE[captype] == "number") continue;
                // Check entryProcedure captype Length
                const currentCapLen = await kernel.getProcCapTypeLen("init", captype as any);
                assert.strictEqual(currentCapLen, 1, `There should be 1 of type: ${CAP_TYPE[captype]}`)
            }
        })

        it('should panic properly', async function () {
            this.timeout(20000);
            let kernel = await newKernelInstance("init", "0xc1912fee45d61c87cc5ea59dae31190fffff232d");
            try {
                await kernel.contract.methods.panic().call();
                assert(false, "method should panic");
            } catch (e) {
                assert(e.message.includes("test-panic"), "the message 'test-panic' should be included in the output");
            }
        })
    })

    describe('validator', function () {
        this.timeout(40_000);
        let kernel: Contract;

        before(async function () {
            let instance = await newKernelInstance("init", "0xc1912fee45d61c87cc5ea59dae31190fffff232d");
            kernel = instance.contract;
        })

        it('should return false when given the null address', async function () {
            this.timeout(20000);
            let rec_validation = await kernel.methods.check_contract('0x0000000000000000000000000000000000000000').call();
            assert.strictEqual(rec_validation, false)
        })

        it('should return panic when given an account addeess (as there is no code)', async function () {
            const accounts = await web3.eth.personal.getAccounts()
            assert(web3.utils.isAddress(accounts[0]), "The example should be a valid address")
            try {
                let rec_validation = await kernel.methods.check_contract(accounts[0]).call();
                throw new Error("check_contract should no succeed");
            } catch (e) {
                // console.log(e)
            }
        })
        it('should return the code size of the kernel', async function () {
            const kernelAddress = kernel.options.address;
            assert(web3.utils.isAddress(kernelAddress), "The kernel address should be a valid address")
            let rec_validation = await kernel.methods.get_code_size(kernelAddress).call();
            assert.strictEqual(typeof rec_validation, "number")
        })

        it('should copy the code of the kernel', async function () {
            const kernelAddress = kernel.options.address;
            assert(web3.utils.isAddress(kernelAddress), "The kernel address should be a valid address")
            const code_size = await kernel.methods.get_code_size(kernelAddress).call();
            const code_hex = await kernel.methods.code_copy(kernelAddress).call();
            const code = web3.utils.hexToBytes(code_hex);
            assert.strictEqual(code.length, code_size, "The code length should be as given by EXTCODESIZE");
        })

        it('should return a boolean when trying to validate the kernel itself', async function () {
            const kernelAddress = kernel.options.address;
            assert(web3.utils.isAddress(kernelAddress), "The kernel address should be a valid address")
            let rec_validation = await kernel.methods.check_contract(kernelAddress).call();
            assert.strictEqual(typeof rec_validation, "boolean");
        })

        it('should copy the code of an example contract', async function () {
            const contract = await deployContract("entry_test", "TestEntryInterface");
            assert(web3.utils.isAddress(contract.address), "The contract address should be a valid address")

            const code_size = await kernel.methods.get_code_size(contract.address).call();
            const code_hex = await kernel.methods.code_copy(contract.address).call();
            const code = web3.utils.hexToBytes(code_hex);
            assert.strictEqual(code_size, code.length, "The code length should be as given by EXTCODESIZE");
        })

        it('should return a boolean when validating an example contract', async function () {
            const contract = await deployContract("entry_test", "TestEntryInterface");
            let rec_validation = await kernel.methods.check_contract(contract.address).call();
            assert.strictEqual(typeof rec_validation, "boolean");
        })

    })

    describe.skip('entry_proc', function () {
        it('should forward call to entry procedure', async function () {
            const accounts = await web3.eth.personal.getAccounts()

            let newProc = await deployContract("entry_test", "TestEntryInterface");
            let kernel = await newKernelInstance("init", newProc.address);

            // Toggle Entry Proc Interface
            await kernel.contract.methods.toggle_syscall();

            // Check Entry Proc is Valid
            let check_entry_result = await newProc.methods.getNum().call();
            assert.equal(check_entry_result, 6);

            let raw_data = newProc.methods.getNum().encodeABI()

            let result = await web3.eth.call({
                from: accounts[0],
                data: raw_data
            })

            assert.equal(result, 6);
        })
    })

})
