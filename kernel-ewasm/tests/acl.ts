const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, EntryCap, WriteCap, RegisterCap, NewCap, AccCallCap} from './utils'
import { Tester, TestContract } from './utils/tester';
import { notEqual } from 'assert';


describe('Access Control List', function () {
    this.timeout(40_000);
    describe('test map', function () {
        it('set and retrieve values', async function () {
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
})