const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, EntryCap, WriteCap, RegisterCap, NewCap, AccCallCap, CHAIN_CONFIG, CallCap, DeleteCap, bufferToHex, hexToBuffer} from './utils'
import { Tester, TestContract } from './utils/tester';
import { notEqual } from 'assert';


describe('Access Control List', function () {
    this.timeout(40_000);
    describe('test ACL boostrap', function () {
        let tester;
        let entry_contract;
        let admin_contract;
        let testAccount;
        let mainAccount;
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

            const testAccountName = "extra_account";
            const testAccountPassword = "extra_password";
            const testAccountRaw = await createAccount(testAccountName, testAccountPassword);
            testAccount = web3.utils.toChecksumAddress(testAccountRaw, web3.utils.hexToNumber(CHAIN_CONFIG.params.networkId));
            await web3.eth.personal.unlockAccount(testAccount, testAccountPassword, null);

            const accounts = await web3.eth.getAccounts();
            mainAccount = accounts[1];
        });

        it('initialise ACL', async function () {

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
        });

        it('register and set group procedures', async function () {

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
        });

        it('remove a user', async function () {
            {
                const m1 = admin_contract.methods.remove_account_group(testAccount).encodeABI();
                const pm1 = tester.interface.methods.proxy(m1).encodeABI();
                await web3.eth.sendTransaction({
                    from: mainAccount,
                    to: tester.interface.address,
                    data: pm1,
                    gas:2_200_000,
                });
            }
            const acl_accounts = await tester.interface.methods.accounts().call();
            assert.deepEqual(acl_accounts, [mainAccount], "Correct accounts should be returned");
            const keys = await tester.kernel.listStorageKeys(1000);
            await checkKernelState(tester.kernel, keys, {
                StorageEnumerableMaps: [
                    "0x3000000000000000000000000000000000000000000000000000000000000000",
                    "0x0000000000000000000000000000000000000000000000000000000000000000",
                ]
            });
        })
    })
})

// In this function we cycle through all of the storage keys which are being
// used and check that they match our expectations of what should be in the
// kernel. It does this by remove each of the well formed items that we expect
// to be found. If either an item that we expect is _not_ well formed, or there
// are entries left over in storage that we have not accounted for, we through
// an error.
async function checkKernelState(kernel, keys, data) {
    // const keys = keysHex.map(hexToBuffer);
    // Build a map of all storage values.
    const storage: Map<string, string> = new Map();
    for (const key of keys) {
        const val = await kernel.getStorageAt(hexToBuffer(key));
        storage.set(key, bufferToHex(val));
    }
    // Remove all the kernel space keys, we won't check those for
    // now.
    for (const keyHex of storage.keys()) {
        const key = hexToBuffer(keyHex);
        if (key[0] === 0xff && key[1] === 0xff && key[2] === 0xff && key[3] === 0xff) {
            storage.delete(bufferToHex(key));
        }
    }
    // Remove all the keys associated with StorageEnumerableMaps
    for (const mapBase of data.StorageEnumerableMaps) {
        removeMap(storage, mapBase);
    }
    for (const [key,val] of storage) {
        console.log(`${key}: ${val}`);
    }
    if (storage.size > 0) {
        throw new Error(`The following storage entries were found in the kernel but not accounted for ${Array.from(storage.keys())}`)
    }
}

// Given a (hex) map from storage keys to storage values, remove each of the
// key/value pairs associated with StorageEnumerable map, roughly checking that
// the StorageEnumerableMap is well formed.
function removeMap(storage, baseHex) {
    const baseKey = hexToBuffer(baseHex);
    // The length value is stored at the baseKey with the length bit set. Does
    // this entry exist?
    baseKey[31] = baseKey[31] | 0b01000000;
    const baseKeyBN = web3.utils.toBN(bufferToHex(baseKey));
    const mapLength = web3.utils.hexToNumber(storage.get(bufferToHex(baseKey)));
    // console.log(`baseKey: ${baseKey}`)
    // console.log(`baseKey: ${bufferToHex(baseKey)}`)
    // console.log(`baseKey: ${hexToBuffer(bufferToHex(baseKey))}`)
    // console.log(`mapLength: ${mapLength}`)
    storage.delete(bufferToHex(baseKey));
    const mapKeys = [];
    // Enumerate all the keys in the map and remove them from our storage map.
    for (let i = 0; i < mapLength; i++) {
        const k = "0x" + web3.utils.toHex(baseKeyBN.add(web3.utils.toBN(1 + i))).slice(2).padStart(64, "0");
        // console.log(`k: ${k}`);
        const val = storage.get(k);
        // console.log(`${i}: ${val}`);
        storage.delete(k);
        mapKeys.push(val);
    }
    // For each of the map keys, remove the presence indicator, and the value.
    // key width in bytes
    for (const mapKeyRaw of mapKeys) {
        const mapKey = hexToBuffer(baseHex);
        // console.log(`mapKeyRaw: ${mapKeyRaw}`)
        const startIndex = hexToBuffer(mapKeyRaw).findIndex(x => x !== 0x00);
        // console.log(`startIndex: ${startIndex}`);
        for (let i = startIndex; i < 32; i++) {
            // shift left 1 byte
            mapKey[i - 1] = hexToBuffer(mapKeyRaw)[i];
        }
        const presenceKey = mapKey.slice();
        presenceKey[31] = presenceKey[31] | 0b10000000;
        // console.log(`mapKey: ${bufferToHex(mapKey)}`)
        // console.log(`presenceKey: ${bufferToHex(presenceKey)}`)
        // Remove map key
        storage.delete(bufferToHex(mapKey));
        // Remove presence key
        storage.delete(bufferToHex(presenceKey));
    }
}
