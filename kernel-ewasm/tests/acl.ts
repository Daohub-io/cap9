const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')
const jayson = require('jayson');

// create a client
const client = jayson.client.http({
  port: 8545
});

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, EntryCap, WriteCap, RegisterCap, NewCap, AccCallCap, CHAIN_CONFIG, CallCap, DeleteCap} from './utils'
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
                new NewCap(0, new CallCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(
                    web3.utils.hexToBytes("0xaa00000000000000000000000000000000000000000000000000000000000000"),
                    web3.utils.hexToBytes("0xffffff0000000000000000000000000000000000000000000000000000000000"),
                )),
                new NewCap(0, new EntryCap()),
            ];
            tester.setFirstEntry("init", new TestContract("acl_entry", "ACLEntryInterface", entryCaps));
            await tester.init();
            const accounts = await web3.eth.getAccounts();
            const mainAccount = accounts[1];
            const procName = "write";
            // Successfuly register a register procedure.
            let regInterface;
            {
                const requestedCaps = [];
                const contractName = "acl_group_5";
                const contractABIName = "ACLGroup5Interface";
                const result = true;
                regInterface = await tester.registerTest(requestedCaps, procName, contractName, contractABIName, result);
            }
            const proc_key = "0x" + web3.utils.fromAscii(procName, 24).slice(2).padStart(64, "0");

            // Create a procedure for Group 5
            await tester.interface.methods.set_group_procedure(5, proc_key).send();
            // Add testAccount to Group 5
            await tester.interface.methods.set_account_group(testAccount, 5).send();
            // Add main account to Group 1 (Admin group)
            await tester.interface.methods.set_account_group(mainAccount, 1).send();
            // When testAccount sends a transaction to the kernel, the message
            // will be transparently passed to the procedure for Group 5. The
            // procedure for Group 5 is now simply a Register Procedure, and
            // should therefore return the number 76 when testNum is called.
            // The way the ACL contract is structured is not using a fallback,
            // but calling the 'proxy' function with the message.

            // There are four steps:
            //   1. Create a message for the final procedure contract we want to
            //      call. This needs to be encoded accoring to the final
            //      contracts ABI.
            //   2. Create a message (proxy_message) for the contract that sits
            //      in between us and the final contract (in this case the ACL
            //      entry procedure). This needs to be encoded with the entry
            //      procedure's ABI.
            //   3. Call the kernel with the proxy message.
            //   4. Decode the result according to the ABI of the final
            //      contract. The kernel and entry procedure pass the return
            //      value from the final procedure back to us unmodified. This
            //      means the return value is encoded accoding to the final
            //      contract, and should be decoded using its ABI.

            // Step 1:
            const message = regInterface.methods.testNum().encodeABI();
            // Step 2:
            const proxy_message = tester.interface.methods.proxy(message).encodeABI();
            // Step 3:
            const return_value = await web3.eth.call({from:testAccount, to:tester.interface.address, data:proxy_message});
            // Step 4:
            const [res,] = web3.eth.abi.decodeParameters(regInterface.jsonInterface.abi.methods.testNum.abiItem.outputs,return_value);
            // Check the value is correct.
            assert.strictEqual(res.toNumber(), 78, "testNum() should return 78");
        })
    })
    describe('test ACL boostrap', function () {
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
                new NewCap(0, new RegisterCap(prefix, cap_key)),
                new NewCap(0, new CallCap(prefix, cap_key)),
                new NewCap(0, new DeleteCap(prefix, cap_key)),
                new NewCap(0, new WriteCap(
                    web3.utils.hexToBytes("0x0000000000000000000000000000000000000000000000000000000000000000"),
                    web3.utils.hexToBytes("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
                )),
                new NewCap(0, new EntryCap()),
            ];

            // Deploy entry contract
            const entry_contract = await deployContract("acl_entry", "ACLEntryInterface", []);
            // Deploy admin contract
            const admin_contract = await deployContract("acl_admin", "ACLAdminInterface", []);
            tester.setFirstEntry("init", new TestContract("acl_bootstrap", "ACLBootstrapInterface", entryCaps));
            await tester.init();
            const accounts = await web3.eth.getAccounts();
            const mainAccount = accounts[1];

            const entry_key = "0x" + web3.utils.fromAscii("entry", 24).slice(2).padStart(64, "0");
            const admin_key = "0x" + web3.utils.fromAscii("admin", 24).slice(2).padStart(64, "0");

            let encoded_cap_list: string[] = entryCaps.reduce((payload, cap) => payload.concat(cap.to_input()), []);
            // Bootstrap the ACL system.
            await tester.interface.methods.init(
                entry_key, // entry key
                entry_contract.address, // entry address
                encoded_cap_list, // entry cap list
                admin_key, // admin key
                admin_contract.address, // admin address
                encoded_cap_list, // admin cap list
                mainAccount // admin account
            ).send();

            // Update the ABI. The entry procedure is now "acl_entry" so we need
            // to use that ABI. We also need to be careful to keep the old
            // address.
            tester.interface = entry_contract;

            const procName = "randomProcName";
            // Successfuly register a procedure for Group 5
            let regInterface;
            {
                const requestedCaps = [];
                const contractName = "acl_group_5";
                const contractABIName = "ACLGroup5Interface";
                const result = true;
                regInterface = await tester.registerTest(requestedCaps, procName, contractName, contractABIName, result);
            }
            const proc_key = "0x" + web3.utils.fromAscii(procName, 24).slice(2).padStart(64, "0");

            // Create a procedure for Group 5
            await tester.interface.methods.set_group_procedure(5, proc_key).send();
            // Add testAccount to Group 5
            await tester.interface.methods.set_account_group(testAccount, 5).send();
            // Add main account to Group 1 (Admin group)
            await tester.interface.methods.set_account_group(mainAccount, 1).send();
            // When testAccount sends a transaction to the kernel, the message
            // will be transparently passed to the procedure for Group 5. The
            // procedure for Group 5 is now simply a Register Procedure, and
            // should therefore return the number 76 when testNum is called.
            // The way the ACL contract is structured is not using a fallback,
            // but calling the 'proxy' function with the message.

            // There are four steps:
            //   1. Create a message for the final procedure contract we want to
            //      call. This needs to be encoded accoring to the final
            //      contracts ABI.
            //   2. Create a message (proxy_message) for the contract that sits
            //      in between us and the final contract (in this case the ACL
            //      entry procedure). This needs to be encoded with the entry
            //      procedure's ABI.
            //   3. Call the kernel with the proxy message.
            //   4. Decode the result according to the ABI of the final
            //      contract. The kernel and entry procedure pass the return
            //      value from the final procedure back to us unmodified. This
            //      means the return value is encoded accoding to the final
            //      contract, and should be decoded using its ABI.

            // Step 1:
            const message = regInterface.methods.testNum().encodeABI();
            // Step 2:
            const proxy_message = tester.interface.methods.proxy(message).encodeABI();
            // Step 3:
            const return_value = await web3.eth.call({from:testAccount, to:tester.interface.address, data:proxy_message});
            // Step 4:
            const [res,] = web3.eth.abi.decodeParameters(regInterface.jsonInterface.abi.methods.testNum.abiItem.outputs,return_value);
            // Check the value is correct.
            assert.strictEqual(res.toNumber(), 78, "testNum() should return 78");
        })
    })
})
