const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, EntryCap, WriteCap, RegisterCap, NewCap, AccCallCap} from '../utils'
import { Tester, TestContract } from '../utils/tester';
import { notEqual } from 'assert';


describe('Account Call Syscall', function () {
    this.timeout(40_000);
    describe('call contract ', function () {
        it('call the external contract', async function () {
            const externalContract = await deployContract("external_contract", "TestExternalInterface");
            const tester = new Tester();
            const prefix = 192;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(0x8000, 2)),
                new NewCap(0, new AccCallCap(true, true, externalContract.address)),
                new NewCap(0, new EntryCap()),
            ];
            tester.setFirstEntry("init", new TestContract("account_call_test", "TestAccountCallInterface", entryCaps));
            await tester.init();
            const value = 0;
            const payload = externalContract.methods.testNum().encodeABI();
            const result = true;
            const return_value = await tester.externalCallTest(externalContract.address, value, payload, result);
            assert.strictEqual(normalize(return_value), normalize(56), "The number should be 56");
        })
        it('fail to call the external contract due to lack of caps', async function () {
            const externalContract = await deployContract("external_contract", "TestExternalInterface");
            const tester = new Tester();
            const prefix = 192;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(0x8000, 2)),
                new NewCap(0, new EntryCap()),
            ];
            tester.setFirstEntry("init", new TestContract("account_call_test", "TestAccountCallInterface", entryCaps));
            await tester.init();
            const value = 0;
            const payload = externalContract.methods.testNum().encodeABI();
            const result = false;
            await tester.externalCallTest(externalContract.address, value, payload, result);
        })
        it('succeed sending value with cap', async function () {
            const externalContract = await deployContract("external_contract", "TestExternalInterface");
            const tester = new Tester();
            const prefix = 192;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(0x8000, 2)),
                new NewCap(0, new AccCallCap(true, true, externalContract.address)),
                new NewCap(0, new EntryCap()),
            ];
            tester.setFirstEntry("init", new TestContract("account_call_test", "TestAccountCallInterface", entryCaps));
            tester.initial_balance = 100;
            await tester.init();
            const value = 5;
            const payload = externalContract.methods.testNum().encodeABI();
            const result = true;
            const return_value = await tester.externalCallTest(externalContract.address, value, payload, result);
            assert.strictEqual(normalize(return_value), normalize(56), "The number should be 56");
        })
        it('fail to send value with insufficient cap', async function () {
            const externalContract = await deployContract("external_contract", "TestExternalInterface");
            const tester = new Tester();
            const prefix = 192;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(0x8000, 2)),
                new NewCap(0, new AccCallCap(true, false, externalContract.address)),
                new NewCap(0, new EntryCap()),
            ];
            tester.setFirstEntry("init", new TestContract("account_call_test", "TestAccountCallInterface", entryCaps));
            tester.initial_balance = 100;
            await tester.init();
            const value = 5;
            const payload = externalContract.methods.testNum().encodeABI();
            const result = false;
            await tester.externalCallTest(externalContract.address, value, payload, result);
        })
        it('succeed calling specific contract with cap', async function () {
            const externalContract = await deployContract("external_contract", "TestExternalInterface");
            const tester = new Tester();
            const prefix = 192;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(0x8000, 2)),
                new NewCap(0, new AccCallCap(false, true, externalContract.address)),
                new NewCap(0, new EntryCap()),
            ];
            tester.setFirstEntry("init", new TestContract("account_call_test", "TestAccountCallInterface", entryCaps));
            await tester.init();
            const value = 0;
            const payload = externalContract.methods.testNum().encodeABI();
            const result = true;
            const return_value = await tester.externalCallTest(externalContract.address, value, payload, result);
            assert.strictEqual(normalize(return_value), normalize(56), "The number should be 56");
        })
        it('fail calling specific contract with insufficient cap', async function () {
            const externalContract = await deployContract("external_contract", "TestExternalInterface");
            const secondExternalContract = await deployContract("external_contract", "TestExternalInterface");
            const tester = new Tester();
            const prefix = 192;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(0x8000, 2)),
                new NewCap(0, new AccCallCap(false, true, secondExternalContract.address)),
                new NewCap(0, new EntryCap()),
            ];
            tester.setFirstEntry("init", new TestContract("account_call_test", "TestAccountCallInterface", entryCaps));
            await tester.init();
            const value = 0;
            const payload = externalContract.methods.testNum().encodeABI();
            const result = false;
            await tester.externalCallTest(externalContract.address, value, payload, result);
        })
        it('succeed calling specific contract with cap (call any and wrong address)', async function () {
            const externalContract = await deployContract("external_contract", "TestExternalInterface");
            const secondExternalContract = await deployContract("external_contract", "TestExternalInterface");
            const tester = new Tester();
            const prefix = 192;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(0x8000, 2)),
                new NewCap(0, new AccCallCap(true, true, secondExternalContract.address)),
                new NewCap(0, new EntryCap()),
            ];
            tester.setFirstEntry("init", new TestContract("account_call_test", "TestAccountCallInterface", entryCaps));
            await tester.init();
            const value = 0;
            const payload = externalContract.methods.testNum().encodeABI();
            const result = true;
            const return_value = await tester.externalCallTest(externalContract.address, value, payload, result);
            assert.strictEqual(normalize(return_value), normalize(56), "The number should be 56");
        })
    })
})
