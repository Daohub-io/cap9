const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, EntryCap, WriteCap, RegisterCap, NewCap, AccCallCap} from './utils'
import { Tester, TestContract } from './utils/tester';
import { notEqual } from 'assert';


describe('Access Control List', function () {
    this.timeout(40_000);
    describe('test map', function () {
        it('call the external contract', async function () {
            const tester = new Tester();
            const prefix = 192;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(0x8000, 2)),
                new NewCap(0, new EntryCap()),
            ];
            tester.setFirstEntry("init", new TestContract("acl", "TestACLInterface", entryCaps));
            await tester.init();
            const procName = "testProc";
            const proc_key = "0x" + web3.utils.fromAscii(procName, 24).slice(2).padStart(64, "0");
            await tester.interface.methods.set_group_procedure(5, proc_key).send();
            const return_value = await tester.interface.methods.get_group_procedure(5).call();
            assert.strictEqual(normalize(proc_key), normalize(return_value), "The procedure name should be correctly set");
        })
    })
})
