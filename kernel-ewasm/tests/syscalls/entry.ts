const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, EntryCap, WriteCap, RegisterCap, NewCap} from '../utils'
import { Tester, TestContract } from '../utils/tester';
import { notEqual } from 'assert';


describe.only('Set Entry Syscall', function () {
    this.timeout(40_000);
    describe('set entry', function () {
        it('set the entry to writer, execute writer', async function () {
            const tester = new Tester();
            const prefix = 192;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(0x8000, 2)),
                new NewCap(0, new EntryCap()),
            ];
            tester.setFirstEntry("init", new TestContract("entry_test", "TestEntryInterface", entryCaps));
            await tester.init();
            const requestedCaps = [new NewCap(0, new WriteCap(0x8000, 2))];
            const procName = "write";
            const contractName = "writer_test";
            const contractABIName = "TestWriterInterface";
            const result = true;
            // Successfuly register a writer procedure
            try {
                await tester.registerTest(requestedCaps, procName, contractName, contractABIName, result);
            } catch (e) {
                console.error("failed to register");
                throw new Error(e);
            }
            try {
                await tester.setEntryTest("write", true);
            } catch (e) {
                console.error("failed to set entry");
                throw new Error(e);
            }
        })
        it('fail to set the entry due to lack of caps', async function () {
            const tester = new Tester();
            const prefix = 192;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(0x8000, 2)),
            ];
            tester.setFirstEntry("init", new TestContract("entry_test", "TestEntryInterface", entryCaps));
            await tester.init();
            const requestedCaps = [new NewCap(0, new WriteCap(0x8000, 2))];
            const procName = "write";
            const contractName = "writer_test";
            const contractABIName = "TestWriterInterface";
            const result = true;
            // Successfuly register a writer procedure
            try {
                await tester.registerTest(requestedCaps, procName, contractName, contractABIName, result);
            } catch (e) {
                console.error("failed to register");
                throw new Error(e);
            }
            try {
                await tester.setEntryTest("write", false);
            } catch (e) {
                console.error("failed to set entry");
                throw new Error(e);
            }
        })
    })
})
