const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, AccCallCap, CallCap, LogCap, DeleteCap, NewCap, RegisterCap, WriteCap } from '../utils'
import { Tester, TestContract } from '../utils/tester';
import { notEqual } from 'assert';


describe('Delete Procedure Syscall', function () {
    this.timeout(40_000);
    describe('#deleteProc', function () {
        it('should register then delete a write procedure', async function () {
            const tester = new Tester();
            const prefix = 192;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new DeleteCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(0x8000, 2)),
            ];
            tester.setFirstEntry("init", new TestContract("delete_test", "TestDeleteInterface", entryCaps));
            await tester.init();
            const requestedCaps = [new NewCap(0, new WriteCap(0x8000, 2))];
            const procName = "write";
            const contractName = "writer_test";
            const contractABIName = "TestWriterInterface";
            const result = true;
            // Successfuly register a writer procedure.
            await tester.registerTest(requestedCaps, procName, contractName, contractABIName, result);
            // Try to delete that procedure.
            await tester.deleteTest(requestedCaps, procName, true);
        })
        it("should fail to delete a procedure which doesn't exit", async function () {
            const tester = new Tester();
            const prefix = 192;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new DeleteCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(0x8000, 2)),
            ];
            tester.setFirstEntry("init", new TestContract("delete_test", "TestDeleteInterface", entryCaps));
            await tester.init();
            const requestedCaps = [new NewCap(0, new WriteCap(0x8000, 2))];
            const procName = "write";
            // Don't register the procedure
            // Try to delete that procedure.
            await tester.deleteTest(requestedCaps, procName, false);
        })
        it("should fail to delete a procedure with no capability", async function () {
            const tester = new Tester();
            const prefix = 192;
            const cap_key = "write";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(0x8000, 2)),
            ];
            tester.setFirstEntry("init", new TestContract("delete_test", "TestDeleteInterface", entryCaps));
            await tester.init();
            const requestedCaps = [new NewCap(0, new WriteCap(0x8000, 2))];
            const procName = "write";
            const contractName = "writer_test";
            const contractABIName = "TestWriterInterface";
            const result = true;
            // Successfuly register a writer procedure.
            await tester.registerTest(requestedCaps, procName, contractName, contractABIName, result);
            // Try to delete that procedure.
            await tester.deleteTest(requestedCaps, procName, false);
        })
        it("should fail to delete a procedure with bad capability", async function () {
            const tester = new Tester();
            const prefix = 192;
            const register_cap_key = "write";
            const delete_cap_key = "read";
            const entryCaps = [
                new NewCap(0, new RegisterCap(prefix, register_cap_key)),
                new NewCap(0, new DeleteCap(prefix, delete_cap_key)),
                new NewCap(0, new WriteCap(0x8000, 2)),
            ];
            tester.setFirstEntry("init", new TestContract("delete_test", "TestDeleteInterface", entryCaps));
            await tester.init();
            const requestedCaps = [new NewCap(0, new WriteCap(0x8000, 2))];
            const procName = "write";
            const contractName = "writer_test";
            const contractABIName = "TestWriterInterface";
            const result = true;
            // Successfuly register a writer procedure.
            await tester.registerTest(requestedCaps, procName, contractName, contractABIName, result);
            // Try to delete that procedure.
            await tester.deleteTest(requestedCaps, procName, false);
        })
    })
})
