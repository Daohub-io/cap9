const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, EntryCap, WriteCap, RegisterCap, NewCap, AccCallCap, CHAIN_CONFIG} from './utils'
import { Tester, TestContract } from './utils/tester';
import { notEqual } from 'assert';


describe('Access Control List', function () {
    this.timeout(40_000);
    describe('test map', function () {
        it('set and retrieve values', async function () {
            const accounts = await web3.eth.personal.getAccounts();
            const tester = new Tester();
            const prefix = 192;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(
                    web3.utils.hexToBytes("0xaa00000000000000000000000000000000000000000000000000000000000000"),
                    web3.utils.hexToBytes("0xffffff0000000000000000000000000000000000000000000000000000000000"),
                )),
                new NewCap(0, new EntryCap()),
            ];
            tester.setFirstEntry("init", new TestContract("acl", "TestACLInterface", entryCaps));
            await tester.init();
            const procName = "testProc";
            const proc_key = "0x" + web3.utils.fromAscii(procName, 24).slice(2).padStart(64, "0");
            await tester.interface.methods.set_group_procedure(5, proc_key).send();
            const return_value = await tester.interface.methods.get_group_procedure(5).call();
            assert.strictEqual(normalize(proc_key), normalize(return_value), "The procedure name should be correctly set");

            await tester.interface.methods.set_account_group(tester.kernel.contract.address, 8).send();
            const group_value = await tester.interface.methods.get_account_group(tester.kernel.contract.address).call();
            assert.strictEqual(normalize(8), normalize(group_value), "The group number should be correctly set");
        })
    })
    describe('test ACL proxy', function () {
        it('set and retrieve values', async function () {
            const testAccountName = "extra_account";
            const testAccountPassword = "extra_password";
            const testAccountRaw = await createAccount(testAccountName, testAccountPassword);
            const testAccount = web3.utils.toChecksumAddress(testAccountRaw, web3.utils.hexToNumber(CHAIN_CONFIG.params.networkId));
            await web3.eth.personal.unlockAccount(testAccount, testAccountPassword, null);

            const tester = new Tester();
            const prefix = 0;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(
                    web3.utils.hexToBytes("0xaa00000000000000000000000000000000000000000000000000000000000000"),
                    web3.utils.hexToBytes("0xffffff0000000000000000000000000000000000000000000000000000000000"),
                )),
                new NewCap(0, new EntryCap()),
            ];
            tester.setFirstEntry("init", new TestContract("acl", "TestACLInterface", entryCaps));
            await tester.init();
            const procName = "write";
            // Successfuly register a register procedure.
            let regInterface;
            {
                const requestedCaps = [];
                const contractName = "register_test";
                const contractABIName = "TestRegisterInterface";
                const result = true;
                regInterface = await tester.registerTest(requestedCaps, procName, contractName, contractABIName, result);
            }
            const proc_key = "0x" + web3.utils.fromAscii(procName, 24).slice(2).padStart(64, "0");

            // Create a procedure for Group 5
            await tester.interface.methods.set_group_procedure(5, proc_key).send();
            // Add testAccount to Group 5
            await tester.interface.methods.set_account_group(testAccount, 5).send();
            // When testAccount sends a transaction to the kernel, the message
            // will be transparently passed to the procedure for Group 5. The
            // procedure for Group 5 is now simply a Register Procedure, and
            // should therefore return the number 76 when testNum is called.
            const message = regInterface.methods.testNum().encodeABI();
            // const return_value = await web3.eth.call({ to: tester.kernel.contract.address, data: message });
            // assert.strictEqual(return_value, 76, "testNum() should return 76");
        })
    })
})
