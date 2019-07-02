const Web3 = require('web3')
const assert = require('assert')
const fs = require('fs')
const jayson = require('jayson');

// create a client
const client = jayson.client.http({
  port: 8545
});
// List storage keys. With parity this can be used to debug the total state of
// the kernel.

import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, EntryCap, WriteCap, RegisterCap, NewCap, AccCallCap, CHAIN_CONFIG, CallCap, DeleteCap, bufferToHex} from './utils'
import { Tester, TestContract } from './utils/tester';
import { notEqual } from 'assert';

async function listStorageKeys(address, n) {
    return new Promise((resolve,reject) => {
        client.request('parity_listStorageKeys', [address, n], function(err, response) {
            if(err) reject(err);
            resolve(response.result);
        });
    });
}

describe('StorgeVec', function () {
    this.timeout(40_000);
    describe('test ACL boostrap', function () {
        it('create vec', async function () {
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

            tester.setFirstEntry("init", new TestContract("storage_vec_test", "StorageVecTestInterface", entryCaps));
            await tester.init();
            const keysBefore = await listStorageKeys(tester.kernel.contract.address, 100);
            await tester.interface.methods.create_vector().send();
            const keysAfter = await listStorageKeys(tester.kernel.contract.address, 100);
            assert.deepEqual(keysBefore, keysAfter, "Storage should be unchanged");
        })
        it('push value', async function () {
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

            tester.setFirstEntry("init", new TestContract("storage_vec_test", "StorageVecTestInterface", entryCaps));
            await tester.init();
            const keys1 = await listStorageKeys(tester.kernel.contract.address, 100);
            await tester.interface.methods.create_vector().send();
            const keys2 = await listStorageKeys(tester.kernel.contract.address, 100);
            assert.deepEqual(keys1, keys2, "Storage should be unchanged");
            const length1 = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]))
                .then(bufferToHex)
                .then(web3.utils.hexToNumber);
            assert.strictEqual(length1, 0, "There should be 0 elements");
            await tester.interface.methods.push_this_proc().send();
            const keys3 = await listStorageKeys(tester.kernel.contract.address, 100);
            const length2 = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]))
                .then(bufferToHex)
                .then(web3.utils.hexToNumber);
            assert.strictEqual(length2, 1, "There should be 1 element");
            const firstKey = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]));
            // console.log(bufferToHex(firstKey));
            assert.strictEqual(firstKey.filter(x=>x !== 0x00).length > 0,true, "The stored value should be non-zero");
            await tester.interface.methods.push_this_proc().send();
            const length3 = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]))
                .then(bufferToHex)
                .then(web3.utils.hexToNumber);
            assert.strictEqual(length3, 2, "There should be 2 elements");
        })
    })
})
