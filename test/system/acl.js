const debug = require('debug')
const assert = require('assert')

const Kernel = artifacts.require('./../TestKernel.sol')
const ACL = artifacts.require('./system/TestACL.sol')
const abi = require('ethereumjs-abi')

const beakerlib = require("../../beakerlib");
const testutils = require("../testutils.js");

const { AbiCoder } = require('web3-eth-abi')
const abiCoder = new AbiCoder()

// const web3_new = new Web3_New('http://localhost:8545')

// // // Valid Contracts
// // const Valid = {
// //     Adder: artifacts.require('test/valid/Adder.sol'),
// //     Multiply: artifacts.require('test/valid/Multiply.sol'),
// //     Divide: artifacts.require('test/valid/Divide.sol'),
// //     SysCallTestWrite: artifacts.require('test/valid/SysCallTestWrite.sol'),
// //     SysCallTestCall: artifacts.require('test/valid/SysCallTestCall.sol'),
// //     FirstNestedCall: artifacts.require('test/valid/NestedCalls/FirstNestedCall.sol'),
// //     SecondNestedCall: artifacts.require('test/valid/NestedCalls/SecondNestedCall.sol'),
// //     ThirdNestedCall: artifacts.require('test/valid/NestedCalls/ThirdNestedCall.sol'),
// //     FourthNestedCall: artifacts.require('test/valid/NestedCalls/FourthNestedCall.sol'),
// //     FifthNestedCall: artifacts.require('test/valid/NestedCalls/FifthNestedCall.sol'),
// //     SixthNestedCall: artifacts.require('test/valid/NestedCalls/SixthNestedCall.sol'),
// //     BasicEntryProcedure: artifacts.require('BasicEntryProcedure.sol'),
// // }

// const TestWrite = artifacts.require('test/TestWrite.sol');

// const Invalid = {
//     Simple: artifacts.require('test/invalid/Simple.sol')
// }

const SELECTOR_ADD_ACCOUNT = "raw_addAccount(address,uint8)";
const SELECTOR_REMOVE_ACCOUNT = "removeAccount(address,uint8)";
const SELECTOR_GET_ACCOUNT_BY_ID = "getAccountById(address)";
const SELECTOR_GET_ACCOUNT_BY_INDEX = "getAccountByIndex(uint8)";

const ACL_DEFAULT_CAPS = beakerlib.Cap.toInput([
    // Account Mapping Cap
    new beakerlib.WriteCap(0x1000, 2 << 21),
    // Account Array Cap
    new beakerlib.WriteCap(0x2000, 256),
    // Account Mapping Cap
    new beakerlib.WriteCap(0x1000, 2 << 21),
    // Account Array Cap
    new beakerlib.WriteCap(0x2000, 256)
]);

class TestACL {

    constructor(web3, kernel, acl) {
        this.kernel = kernel;
        this.acl = acl;
        this.web3 = web3;
    }

    async register(caps = ACL_DEFAULT_CAPS) {
        const { kernel, acl } = this;
        return await kernel.registerAnyProcedure("ACL", acl.address, caps);
    }

    async createGroup(procId) {
        const { kernel, web3 } = this;

        const functionSelector = "createGroup(bytes24)";
        const functionSelectorHash = web3.sha3(functionSelector).slice(2, 10);
        const inputData = web3.fromAscii("ACL".padEnd(24, "\0"))
            + functionSelectorHash
            + web3.fromAscii(procId.padEnd(24, "\0")).slice(2).padEnd(32 * 2, 0)

        const valueXRawRaw = await web3.eth.call({ to: kernel.address, data: inputData });
        const value = web3.toBigNumber(valueXRawRaw);

        const tx = await kernel.sendTransaction({ data: inputData });
        return { tx, groupIndex: value };
    }

    async getGroupByIndex(groupIndex) {
        const { kernel, web3 } = this;

        const functionSelector = "getGroupByIndex(uint8)";
        const functionSelectorHash = web3.sha3(functionSelector).slice(2, 10);
        const inputData2 = web3.fromAscii("ACL".padEnd(24, "\0"))
            + functionSelectorHash
            + web3.toHex(groupIndex).slice(2).padStart(32 * 2, 0) // the amount argument for call (32 bytes)

        const valueXRawRaw = await web3.eth.call({ to: kernel.address, data: inputData2 });
        return abiCoder.decodeParameters([{ name: 'procId', type: 'bytes24' }, { name: 'accountsLen', type: 'uint8' }, { name: 'groupIndex', type: 'uint8' }], valueXRawRaw)
    }

    async removeGroup(procId) {
        const { kernel, web3 } = this;

        const functionSelector = "removeGroup(bytes24)";
        const functionSelectorHash = web3.sha3(functionSelector).slice(2, 10);
        const inputData = web3.fromAscii("ACL".padEnd(24, "\0"))
            + functionSelectorHash
            + web3.fromAscii(procId.padEnd(24, "\0")).slice(2).padEnd(32 * 2, 0)

        const valueXRawRaw = await web3.eth.call({ to: kernel.address, data: inputData });
        const value = web3.toBigNumber(valueXRawRaw);

        const tx = await kernel.sendTransaction({ data: inputData });
        return { tx, groupIndex: value };
    }

}

contract.only('ACL', function (accounts) {
    describe('#_createGroup(bytes24)', function () {
        it('should push new group', async function () {
            const kernel = await testutils.deployTestKernel();
            const acl = await testutils.deployedTrimmed(ACL)
            const testACL = new TestACL(web3, kernel, acl)
            const tx1 = await testACL.register();

            // Create Group FOO
            const foo_res = await testACL.createGroup("FOO");
            assert.equal(foo_res.groupIndex, 0)

            // Create Group BAR
            const bar_res = await testACL.createGroup("BAR");
            assert.equal(bar_res.groupIndex, 1)

            // Get Groups
            const foo = await testACL.getGroupByIndex(0)
            assert.equal(foo.procId, web3.fromAscii("FOO".padEnd(24, "\0")))
            const bar = await testACL.getGroupByIndex(1)
            assert.equal(bar.procId, web3.fromAscii("BAR".padEnd(24, "\0")))
        })

    })

    describe('#_removeGroup(bytes24)', function () {
        it('should remove group', async function() {
            const kernel = await testutils.deployTestKernel();
            const acl = await testutils.deployedTrimmed(ACL)
            const testACL = new TestACL(web3, kernel, acl)
            const tx1 = await testACL.register();

            // Create Group FOO
            const foo_res = await testACL.createGroup("FOO");
            assert.equal(foo_res.groupIndex, 0)

            // Create Group BAR
            const bar_res = await testACL.createGroup("BAR");
            assert.equal(bar_res.groupIndex, 1)

            // Remove Group FOO
            const foo_res_removed = await testACL.removeGroup("FOO");
            assert.equal(foo_res_removed.groupIndex, 0)

            // GROUP BAR Should be moved to index 0
            const bar = await testACL.getGroupByIndex(0)
            assert.equal(bar.groupIndex, web3.fromAscii("BAR".padEnd(24, "\0")))
        })
    })

    //     it('S() should fail when not given cap', async function () {

    //         const kernel = await testutils.deployTestKernel();

    //         const SysCallTestWrite = await testutils.deployedTrimmed(Valid.SysCallTestWrite);
    //         const simpleTest = await testutils.deployedTrimmed(Valid.Multiply);
    //         const tx1 = await kernel.registerProcedure("SysCallTestWrite", SysCallTestWrite.address, []);
    //         const tx2 = await kernel.registerProcedure("Simple", simpleTest.address, []);

    //         const newValue1 = await kernel.testGetter.call();
    //         assert.equal(newValue1.toNumber(), 3, "The value should be 3 before the execution");


    //         // Procedure keys must occupay the first 24 bytes, so must be
    //         // padded
    //         const functionSelector = "S()";
    //         // const functionSelectorHash = web3.sha3(functionSelector);
    //         const functionSelectorHash = "4be1c796"
    //         const inputData = web3.fromAscii("SysCallTestWrite".padEnd(24,"\0")) + functionSelectorHash;
    //         const tx3 = await kernel.sendTransaction({data: inputData});

    //         // for (const log of tx3.receipt.logs) {
    //         //     if (log.topics.length > 0) {
    //         //         console.log(`Log: ${web3.toAscii(log.topics[0])} - ${log.data} - ${web3.toAscii(log.data)}`);
    //         //     } else {
    //         //         console.log(`Log: ${log.topics[0]} - ${web3.toAscii(log.data)} - ${log.data}`);
    //         //     }
    //         // }

    //         // The log value is 32 bytes log so we pad it out with nulls
    //         const expectedLogValue = "BasicEntryProcedureFallback".padEnd(32,'\0');
    //         // Should be trimEnd, but I left it as trim in case you don't
    //         // have node 10

    //         const newValue4 = await kernel.testGetter.call();
    //         assert.equal(newValue4.toNumber(), 3, "The value should still be 3 after the execution");
    //     })
    //     it('S() should fail when trying to write to an address below its cap', async function () {

    //         const kernel = await testutils.deployTestKernel();

    //         const capArraySysCallTest = beakerlib.Cap.toInput([
    //             new beakerlib.WriteCap(0x8500,2),
    //             new beakerlib.WriteCap(0x8001,0)
    //         ]);
    //         const SysCallTestWrite = await testutils.deployedTrimmed(Valid.SysCallTestWrite);
    //         const simpleTest = await testutils.deployedTrimmed(Valid.Multiply);
    //         const tx1 = await kernel.registerProcedure("SysCallTestWrite", SysCallTestWrite.address, capArraySysCallTest);
    //         const tx2 = await kernel.registerProcedure("Simple", simpleTest.address, []);

    //         const newValue1 = await kernel.testGetter.call();
    //         assert.equal(newValue1.toNumber(), 3, "The value should be 3 before the execution");


    //         // Procedure keys must occupay the first 24 bytes, so must be
    //         // padded
    //         const functionSelector = "S()";
    //         // const functionSelectorHash = web3.sha3(functionSelector);
    //         const functionSelectorHash = "4be1c796"
    //         const inputData = web3.fromAscii("SysCallTestWrite".padEnd(24,"\0")) + functionSelectorHash;
    //         const tx3 = await kernel.sendTransaction({data: inputData});

    //         // for (const log of tx3.receipt.logs) {
    //         //     if (log.topics.length > 0) {
    //         //         console.log(`Log: ${web3.toAscii(log.topics[0])} - ${log.data} - ${web3.toAscii(log.data)}`);
    //         //     } else {
    //         //         console.log(`Log: ${log.topics[0]} - ${web3.toAscii(log.data)} - ${log.data}`);
    //         //     }
    //         // }

    //         // The log value is 32 bytes log so we pad it out with nulls
    //         const expectedLogValue = "BasicEntryProcedureFallback".padEnd(32,'\0');
    //         // Should be trimEnd, but I left it as trim in case you don't
    //         // have node 10

    //         const newValue4 = await kernel.testGetter.call();
    //         assert.equal(newValue4.toNumber(), 3, "The value should still be 3 after the execution");
    //     })
    // })
})