const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, EntryCap, WriteCap, RegisterCap, NewCap, AccCallCap, CHAIN_CONFIG, CallCap, DeleteCap, bufferToHex} from './utils'
import { Tester, TestContract } from './utils/tester';
import { notEqual } from 'assert';


describe('Access Control List', function () {
    this.timeout(40_000);
    describe('test ACL boostrap', function () {
        let tester;
        let entry_contract;
        let admin_contract;
        const prefix = 0;
        const cap_key = "write";
        const entryCaps = [
            new NewCap(0, new RegisterCap(prefix, cap_key)),
            new NewCap(1, new RegisterCap(prefix, cap_key)),
            new NewCap(0, new CallCap(prefix, cap_key)),
            new NewCap(0, new DeleteCap(prefix, cap_key)),
            new NewCap(0, new WriteCap(
                web3.utils.hexToBytes("0x0000000000000000000000000000000000000000000000000000000000000000"),
                web3.utils.hexToBytes("0x1000000000000000000000000000000000000000000000000000000000000000"),
            )),
            new NewCap(1, new WriteCap(
                web3.utils.hexToBytes("0x3000000000000000000000000000000000000000000000000000000000000000"),
                web3.utils.hexToBytes("0x1000000000000000000000000000000000000000000000000000000000000000"),
            )),
            new NewCap(0, new EntryCap()),
        ];

        this.beforeAll(async function () {
            tester = new Tester();

            // Deploy entry contract
            entry_contract = await deployContract("acl_entry", "ACLEntryInterface", []);
            // Deploy admin contract
            admin_contract = await deployContract("acl_admin", "ACLAdminInterface", []);
            tester.setFirstEntry("init", new TestContract("acl_bootstrap", "ACLBootstrapInterface", entryCaps));
            await tester.init();
        });

        it('set and retrieve values', async function () {
            const testAccountName = "extra_account";
            const testAccountPassword = "extra_password";
            const testAccountRaw = await createAccount(testAccountName, testAccountPassword);
            const testAccount = web3.utils.toChecksumAddress(testAccountRaw, web3.utils.hexToNumber(CHAIN_CONFIG.params.networkId));
            await web3.eth.personal.unlockAccount(testAccount, testAccountPassword, null);

            const accounts = await web3.eth.getAccounts();
            const mainAccount = accounts[1];

            const entry_key = "0x" + web3.utils.fromAscii("entry", 24).slice(2).padStart(64, "0");
            const admin_key = "0x" + web3.utils.fromAscii("admin", 24).slice(2).padStart(64, "0");

            const encoded_cap_list: string[] = entryCaps.reduce((payload, cap) => payload.concat(cap.to_input()), []);
            // Bootstrap the ACL system.
            await tester.interface.methods.init(
                entry_key, // entry key
                entry_contract.address, // entry address
                encoded_cap_list, // entry cap list
                admin_key, // admin key
                admin_contract.address, // admin address
                encoded_cap_list, // admin cap list
                mainAccount // admin account
            ).send({gas:2_000_000});
            // Update the ABI. The entry procedure is now "acl_entry" so we need
            // to use that ABI. We also need to be careful to keep the old
            // address.
            tester.interface = entry_contract;

            const n_accounts = await tester.interface.methods.n_accounts().call().then(x=>x.toNumber());
            assert.strictEqual(n_accounts, 1, "There should be one account");

            const procName = "randomProcName";
            // Successfuly register a procedure for Group 5
            const proc_key = "0x" + web3.utils.fromAscii(procName, 24).slice(2).padStart(64, "0");
            let regInterface;
            {
                const requestedCaps = [];
                const contractName = "acl_group_5";
                const contractABIName = "ACLGroup5Interface";
                const cap_index = 0;
                const contract = await deployContract(contractName, contractABIName);
                const encodedRequestedCaps = requestedCaps.reduce((payload, cap) => payload.concat(cap.to_input()), []);
                const message = admin_contract.methods.regProc(cap_index, proc_key, contract.address, encodedRequestedCaps).encodeABI();
                const proxy_message = tester.interface.methods.proxy(message).encodeABI();
                await web3.eth.sendTransaction({ to: tester.kernel.contract.address, data: proxy_message, gas:2_100_000});
                regInterface = contract;
            }

            // Swith to the Admin ABI
            tester.interface = admin_contract;
            // Create a procedure for Group 5
            {
                const m1 = admin_contract.methods.set_group_procedure(5, proc_key).encodeABI();
                const pm1 = tester.interface.methods.proxy(m1).encodeABI();
                await web3.eth.sendTransaction({
                    from: mainAccount,
                    to: tester.interface.address,
                    data: pm1,
                    gas: 6_000_000,
                });
            }
            // Add testAccount to Group 5
            {
                const m1 = admin_contract.methods.set_account_group(testAccount, 5).encodeABI();
                const pm1 = tester.interface.methods.proxy(m1).encodeABI();
                await web3.eth.sendTransaction({
                    from: mainAccount,
                    to: tester.interface.address,
                    data: pm1,
                    gas:2_200_000,
                });
            }
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
            // const procs = await tester.kernel.getProcedures();
            // procs.forEach(proc => {
            //     console.log(proc.toString())
            // });
            // const keys = await tester.kernel.listStorageKeys(100);
            // console.log(keys)
            // Switch to entry inteface
            tester.interface = entry_contract;
            const acl_accounts = await tester.interface.methods.accounts().call();
            const acl_procedures = await tester.interface.methods.procedures().call();
            assert.deepEqual(acl_accounts, [mainAccount, testAccount], "Correct accounts should be returned");
        })
    })
})
