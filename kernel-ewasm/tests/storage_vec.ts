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
    describe('test vector', function () {
        let tester;
        this.beforeAll(async function () {
            tester = new Tester();
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
        });

        it('create vec', async function () {
            const keysBefore = await listStorageKeys(tester.kernel.contract.address, 100);
            await tester.interface.methods.create_vector().send();
            const keysAfter = await listStorageKeys(tester.kernel.contract.address, 100);
            assert.deepEqual(keysBefore, keysAfter, "Storage should be unchanged");
        });

        it('push value first value', async function () {
            const length1 = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]))
                .then(bufferToHex)
                .then(web3.utils.hexToNumber);
            assert.strictEqual(length1, 0, "There should be 0 elements");
            await tester.interface.methods.push_num(85).send();
            const keys3 = await listStorageKeys(tester.kernel.contract.address, 100);
            const length2 = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]))
                .then(bufferToHex)
                .then(web3.utils.hexToNumber);
            assert.strictEqual(length2, 1, "There should be 1 element");
            const firstKey1 = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]))
                .then(bufferToHex)
                .then(web3.utils.hexToNumber);
            assert.strictEqual(firstKey1, 85, "The first stored value should be 85");
        });

        it('push value second value', async function () {
            await tester.interface.methods.push_num(95).send();
            const length3 = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]))
                .then(bufferToHex)
                .then(web3.utils.hexToNumber);
            const firstKey2 = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]))
                .then(bufferToHex)
                .then(web3.utils.hexToNumber);
            assert.strictEqual(firstKey2, 85, "The first stored value should be still be 85");
            const secondKey1 = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2]))
                .then(bufferToHex)
                .then(web3.utils.hexToNumber);
            assert.strictEqual(secondKey1, 95, "The second stored value should be be 95");
            assert.strictEqual(length3, 2, "There should be 2 elements");
        });

        it('push and pop third value', async function () {
            // The number randomly chosen to be out test element.
            const testValue = 105;
            // At the start of this test the vector should have 2 elements.
            const length1 = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]))
                .then(bufferToHex)
                .then(web3.utils.hexToNumber);
            assert.strictEqual(length1, 2, "There should be 2 elements");
            // Push another element
            await tester.interface.methods.push_num(testValue).send();
            // The vector should now have 3 elements.
            const length2 = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]))
                .then(bufferToHex)
                .then(web3.utils.hexToNumber);
            assert.strictEqual(length2, 3, "There should be 3 elements");
            // The third element should be the value we just pushed.
            const pushedElement = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3]))
                .then(bufferToHex)
                .then(web3.utils.hexToNumber);
            assert.strictEqual(pushedElement, testValue, "The third stored value be still be the testValue");
            // Pop the last element. We perform a call (to actually get the
            // value), and a send (to make the change in the vector permanent).
            const poppedVal = await tester.interface.methods.pop_num().call();
            await tester.interface.methods.pop_num().send();
            assert.strictEqual(poppedVal.toNumber(), testValue, "The popped value should be the testValue");
            const length3 = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]))
                .then(bufferToHex)
                .then(web3.utils.hexToNumber);
            assert.strictEqual(length3, 2, "There should be 2 elements");

            // Get the value in the storage space where the element was.
            const pushedElementSpace = await tester.kernel.getStorageAt(Uint8Array.from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3]))
                .then(bufferToHex);
            assert.strictEqual(pushedElementSpace, "0x0000000000000000000000000000000000000000000000000000000000000000", "The popped value should have been cleared");
        });

        it('sum over iterator', async function () {
            const result = await tester.interface.methods.sum().call();
            // The sum should be the first two numbers, but not the third (as it
            // was popped).
            assert.strictEqual(result.toNumber(), 85+95, "The sum should be correct");
        });
    })
})
