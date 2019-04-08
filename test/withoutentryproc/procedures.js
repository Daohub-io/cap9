const debug = require('debug')
const assert = require('assert')

const Kernel = artifacts.require('./TestKernel.sol')
const abi = require('ethereumjs-abi')

const beakerlib = require("../../beakerlib");
const testutils = require("../testutils.js");

const CAP_TYPE = beakerlib.CAP_TYPE;

// Valid Contracts
const Valid = {
    Adder: artifacts.require('test/valid/Adder.sol'),
    Multiply: artifacts.require('test/valid/Multiply.sol'),
    Divide: artifacts.require('test/valid/Divide.sol'),
    SysCallTestWrite: artifacts.require('test/valid/SysCallTestWrite.sol'),
    Simple: artifacts.require('test/valid/Simple.sol'),
    SysCallTestLog: artifacts.require('test/valid/SysCallTestLog.sol'),
}

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

// Test utility functions
function isNullAddress(address) {
    return address === "0x0000000000000000000000000000000000000000";
}

const testDebug = debug('test:Factory')
const testAccount = 0;

// For use with parity test account
// web3.eth.defaultAccount = "0x00a329c0648769a73afac7f9381e08fb43dbea72";
// throw new Error("the test")
contract('Kernel without entry procedure', function (accounts) {
    describe('.listProcedures()', function () {
        it('should return nothing if zero procedures', async function () {
            let kernel = await Kernel.new();

            let procedures = await kernel.listProcedures.call();
            assert.equal(procedures.length, 0);
        })
        it('should return existing procedure keys', async function () {
            let kernel = await Kernel.new();

            const testAdder = await testutils.deployedTrimmed(Valid.Adder);
            let [err, address] = await kernel.registerProcedure.call('TestAdder', testAdder.address, []);
            let tx1 = await kernel.registerProcedure('TestAdder', testAdder.address, []);

            let procedures = await kernel.listProcedures.call();
            assert.equal(procedures.length, 1);
        });
        it('should return a list of procedures which can be retrieved', async function () {
            const kernel = await Kernel.new();
            const speccedProcedures = [
                ["TestAdder", Valid.Adder],
                ["TestDivider", Valid.Divide],
                ["TestMultiplier", Valid.Multiply]
            ];

            for (const proc of speccedProcedures) {
                const contract = await testutils.deployedTrimmed(proc[1]);
                await kernel.registerProcedure(proc[0], contract.address, []);
            }

            const proceduresRaw = await kernel.listProcedures.call();
            const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));

            // Test that the number of procedures stored is the same as the
            // number of procedures created
            assert.equal(procedures.length, speccedProcedures.length);
            // Cycle through each of the listed procedures
            for (const i in procedures) {
                // Test that the order and indexing of procedures is the same
                assert.equal(speccedProcedures[i][0], procedures[i])
                // Retrieve the listed procedure adress
                const address = await kernel.getProcedure.call(procedures[i]);
                // Check the address is correct
                assert(web3.isAddress(address), `Procedure Address (${address}) is a real address`);
                assert(!isNullAddress(address), `Procedure Address (${address}) is not null`);
                // Check that the deployed code is the same as that sent
                const code = web3.eth.getCode(address);
                assert.equal(testutils.trimSwarm(speccedProcedures[i][1].deployedBytecode), code);
            }
        });
    })
    describe('.getProcedure()', function () {
        it('should return a non-zero address iff procedure exists', async function () {
            let kernel = await Kernel.new();

            // Create "TestAdder"
            // Find the address (ephemerally)
            const testAdder = await testutils.deployedTrimmed(Valid.Adder);
            let [err, creationAddress] = await kernel.registerProcedure.call('TestAdder', testAdder.address, []);
            assert(web3.isAddress(creationAddress), `Procedure Creation Address (${creationAddress}) is a real address`);
            assert(!isNullAddress(creationAddress), `Procedure Creation Address (${creationAddress}) is not null`);

            // Carry out the creation
            let tx1 = await kernel.registerProcedure('TestAdder', testAdder.address, []);

            // Get the procedure
            let address = await kernel.getProcedure.call("TestAdder");
            assert(web3.isAddress(address), `Procedure Address (${address}) is a real address`);
            assert(!isNullAddress(address), `Procedure Address (${address}) is not null`);

            assert.equal(creationAddress, address);
        });
        it('should return a zero address iff procedure does not exist', async function () {
            let kernel = await Kernel.new();
            // No procedures exist yet (nor does "TestAdder")
            let address = await kernel.getProcedure.call('TestAdder');
            assert(web3.isAddress(address), `Procedure Address (${address}) is a real address`)
            assert(isNullAddress(address), `Procedure Address (${address}) is null`)
        });
    })

    describe('.registerProcedure()', function () {
        it('should create valid procedure', async function () {
            let kernel = await Kernel.new();
            const procedureName = "TestAdder";
            const testAdder = await testutils.deployedTrimmed(Valid.Adder);
            let [err, address] = await kernel.registerProcedure.call(procedureName, testAdder.address, []);
            let tx1 = await kernel.registerProcedure(procedureName, testAdder.address, []);

            assert(web3.isAddress(address), `Procedure Address (${address}) is a real address`)
            assert(!isNullAddress(address), 'Procedure Address is not null')

            // Check that the code works
            let adder = Valid.Adder.at(address);
            let result;
            try {
                result = await adder.add.call(1, 1);
            } catch (e) {
                throw new Error("add.call failed");
            }
            assert.equal(result, 2)

            // The returned code should be the same as the sent code
            const code = web3.eth.getCode(address);
            assert.equal(testutils.trimSwarm(Valid.Adder.deployedBytecode), code);
        });
        it('should create valid procedure (max key length)', async function () {
            const kernel = await Kernel.new();
            const name = "start1234567890123456end";
            assert.equal(name.length, 24);
            const testAdder = await testutils.deployedTrimmed(Valid.Adder);
            const [err, address] = await kernel.registerProcedure.call(name, testAdder.address, []);
            const tx1 = await kernel.registerProcedure(name, testAdder.address, []);

            assert(web3.isAddress(address), `Procedure Address (${address}) should be a real address`)
            assert(!isNullAddress(address), 'Procedure Address should not be null')

            const adder = Valid.Adder.at(address);
            assert.equal(await adder.add.call(1, 1), 2)

            // The returned code should be the same as the sent code
            const code = web3.eth.getCode(address);
            assert.equal(testutils.trimSwarm(Valid.Adder.deployedBytecode), code);

            // The address should be gettable (TODO)
            // The correct name should be in the procedures table
            const proceduresRaw = await kernel.listProcedures.call();
            const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
            assert(procedures.includes(name), "The correct name is in the procedures table");
        });

        it('should create 2 valid procedures', async function () {
            const kernel = await Kernel.new();

            const proceduresRaw1 = await kernel.listProcedures.call();
            const name = "start1234567890123456end";
            assert.equal(name.length, 24);
            const testAdder = await testutils.deployedTrimmed(Valid.Adder);
            const [err, address] = await kernel.registerProcedure.call(name, testAdder.address, []);
            const tx1 = await kernel.registerProcedure(name, testAdder.address, []);

            assert(web3.isAddress(address), `Procedure Address (#1) (${address}) should be a real address`)
            assert(!isNullAddress(address), 'Procedure Address (#1) should not be null')

            const adder = Valid.Adder.at(address);
            assert.equal(await adder.add.call(1, 1), 2)

            // The returned code should be the same as the sent code
            const code = web3.eth.getCode(address);
            assert.equal(testutils.trimSwarm(Valid.Adder.deployedBytecode), code);
            const name2 = "ByAnyOtherName";
            const testAdder2 = await testutils.deployedTrimmed(Valid.Adder);
            const [err2, address2] = await kernel.registerProcedure.call(name2, testAdder2.address, []);
            const tx2 = await kernel.registerProcedure(name2, testAdder2.address, []);

            assert(web3.isAddress(address2), `Procedure Address (#2) (${address2}) should be a real address`)
            assert(!isNullAddress(address2), 'Procedure Address (#2) should not be null')

            // The returned code should be the same as the sent code
            const code2 = web3.eth.getCode(address2);
            assert.equal(testutils.trimSwarm(Valid.Adder.deployedBytecode), code2);

            // The address should be gettable (TODO)
            // The correct name should be in the procedures table
            const proceduresRaw = await kernel.listProcedures.call();
            const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
            assert(procedures.includes(name), "The correct name is in the procedures table");
        });

        // TODO: what is an invalid payload?
        it('should reject invalid payload')

        describe('should reject invalid key', function () {

            it('zero length', async function () {
                let kernel = await Kernel.new();

                const testAdder = await testutils.deployedTrimmed(Valid.Adder);
                let [err, creationAddress] = await kernel.registerProcedure.call('', testAdder.address, []);
                assert.equal(err, 1);
                assert(web3.isAddress(creationAddress), `Procedure Creation Address (${creationAddress}) is a real address`)
                assert(isNullAddress(creationAddress), `Procedure Creation Address (${creationAddress}) is null`)

                const proceduresRaw = await kernel.listProcedures.call();
                const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                assert.equal(procedures.length, 0);
                assert(!procedures.includes(''))

                const address = await kernel.getProcedure.call('');
                assert(web3.isAddress(address), `Procedure Address (${address}) is a real address`)
                assert(isNullAddress(address), 'Procedure Address is null')
            });

            it('duplicate procedure key', async function () {
                const kernel = await Kernel.new();
                const name = "TestAdder";
                const testAdder = await testutils.deployedTrimmed(Valid.Adder);
                // This is the first time the procedure is added
                const [err1, address1] = await kernel.registerProcedure.call(name, testAdder.address, []);
                const tx1 = await kernel.registerProcedure(name, testAdder.address, []);

                // This is the second time the procedure is added
                const testMultiply = await testutils.deployedTrimmed(Valid.Multiply);
                const [err2, address2] = await kernel.registerProcedure.call(name, testMultiply.address, []);
                const tx2 = await kernel.registerProcedure(name, testMultiply.address, []);
                assert.equal(err2.toNumber(), 4);

                const proceduresRaw = await kernel.listProcedures.call();
                const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
                assert.equal(procedures.length, 1);

                const address = await kernel.getProcedure.call(name);
                assert.equal(address, address1);
                assert.notEqual(address, address2);
                assert(web3.isAddress(address), `Procedure Address (${address}) is a real address`)
                assert(!isNullAddress(address), 'Procedure Address is not null')

                // The returned code should be the same as the original code
                const code = web3.eth.getCode(address);
                assert.equal(testutils.trimSwarm(Valid.Adder.deployedBytecode), code);
            });
        })
    })

    describe('.deleteProcedure()', function () {
        it('should return error if procedure key does not exist(3)', async function () {
            const kernel = await Kernel.new();
            const [err, deleteAddress] = await kernel.deleteProcedure.call('test');
            assert.equal(err, 2);
        });
        it('should return deleted procedure address if procedure key is valid', async function () {
            const kernel = await Kernel.new();
            const testAdder = await testutils.deployedTrimmed(Valid.Adder);
            const [err1, address] = await kernel.registerProcedure.call("test", testAdder.address, []);
            assert.equal(err1, 0);
            const tx1 = await kernel.registerProcedure('test', testAdder.address, []);
            const code = web3.eth.getCode(address);
            const codeAsNumber = web3.toBigNumber(code);
            // There should be some code at this address now
            // (here it is returned as a hex, we test the string because we
            // want to also be sure it is encoded as such)
            assert.notEqual(code, "0x0");

            const [err2, deleteAddress] = await kernel.deleteProcedure.call('test');
            assert.equal(err2, 0);

            const tx2 = await kernel.deleteProcedure('test');

            assert.equal(address, deleteAddress);
        });

        it('should remove the procedure from the list on deletion', async function () {
            const kernel = await Kernel.new();

            const procedureName = "test";
            const testAdder = await testutils.deployedTrimmed(Valid.Adder);
            const [err1, address] = await kernel.registerProcedure.call(procedureName, testAdder.address, []);
            assert.equal(err1, 0);
            const tx1 = await kernel.registerProcedure(procedureName, testAdder.address, []);
            const code = web3.eth.getCode(address);
            const codeAsNumber = web3.toBigNumber(code);

            // There should be some code at this address now
            // (here it is returned as a hex, we test the string because we
            // want to also be sure it is encoded as such)
            assert.notEqual(code, "0x0");
            assert.notEqual(code, "0x");

            const [err2, deleteAddress] = await kernel.deleteProcedure.call(procedureName);
            assert.equal(err2, 0);
            const retrievedAddress1 = await kernel.getProcedure(procedureName);
            assert(!isNullAddress(retrievedAddress1), "The key should be retrievable")

            const tx2 = await kernel.deleteProcedure('test');

            assert.equal(address, deleteAddress);
            const retrievedAddress = await kernel.getProcedure(procedureName);
            // console.log(retrievedAddress)
            assert(isNullAddress(retrievedAddress), "The key should not be retrievable")

            const proceduresRaw = await kernel.listProcedures.call();
            const procedures = proceduresRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
            assert(!procedures.includes(procedureName), "The procedure name should no longer be included in the procedure table")
        })

        it('should remove the procedure from the list on deletion (multiple)', async function () {
            const kernel = await Kernel.new();

            const procedureName = "test";
            const testAdder = await testutils.deployedTrimmed(Valid.Adder);
            const [err1, address] = await kernel.registerProcedure.call(procedureName, testAdder.address, []);
            assert.equal(err1, 0);
            const tx1 = await kernel.registerProcedure(procedureName, testAdder.address, []);
            const code = web3.eth.getCode(address);
            const codeAsNumber = web3.toBigNumber(code);

            // There should be some code at this address now
            // (here it is returned as a hex, we test the string because we
            // want to also be sure it is encoded as such)
            assert.notEqual(code, "0x0");
            assert.notEqual(code, "0x");

            await kernel.registerProcedure(procedureName+"1", testAdder.address, []);
            await kernel.registerProcedure(procedureName+"2", testAdder.address, []);
            await kernel.registerProcedure(procedureName+"3", testAdder.address, []);

            const proceduresBeforeRaw = await kernel.listProcedures.call();
            const proceduresBefore = proceduresBeforeRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));
            const [err2, deleteAddress] = await kernel.deleteProcedure.call(procedureName);
            assert.equal(err2, 0);
            const tx2 = await kernel.deleteProcedure(procedureName);
            const proceduresAfterRaw = await kernel.listProcedures.call();
            const proceduresAfter = proceduresAfterRaw.map(web3.toAscii).map(s => s.replace(/\0.*$/, ''));


            assert.equal(address, deleteAddress);
            const retrievedAddress = await kernel.getProcedure(procedureName);
            assert(isNullAddress(retrievedAddress), "The key be able to be retrieved")

            assert(!proceduresAfter.includes(procedureName), "The procedure name should no longer be included in the procedure table")
            assert(proceduresAfter.includes(procedureName+"1"), "This procedure name should still be included in the procedure table")
            assert(proceduresAfter.includes(procedureName+"2"), "This procedure name should still be included in the procedure table")
            assert(proceduresAfter.includes(procedureName+"3"), "This procedure name should still be included in the procedure table")

            // The last procedure should have been moved to the beginning
            // first we find the original position of the deleted procedure
            const origPos = proceduresBefore.findIndex((s)=>s===procedureName);
            // then we check that the last procedure is now in that location
            assert.equal(proceduresAfter[origPos],proceduresBefore[proceduresBefore.length-1], "The last procedure should have been moved to the beginning")
        })

        describe('should not have side-effects', function() {

            it('removing then registering a new proc with same name + capabilities', async function() {
                const kernel = await Kernel.new();
                const table0 = await kernel.returnRawProcedureTable.call()

                const cap1 = new beakerlib.LogCap([]);
                const cap2 = new beakerlib.LogCap(["0xdeadbeef"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                const procedureName = "test";
                const testAdder = await testutils.deployedTrimmed(Valid.Adder);

                await kernel.registerProcedure.call(procedureName, testAdder.address, capArray);
                await kernel.registerProcedure(procedureName, testAdder.address, capArray);
                const table1 = await kernel.returnRawProcedureTable.call()

                // Delete Procedure
                await kernel.deleteProcedure.call(procedureName);
                await kernel.deleteProcedure(procedureName);
                const table2 = await kernel.returnRawProcedureTable.call()
                assert.deepEqual(table0, table2, 'Procedure Tables should be equal after deletion')

                const retrievedAddress = await kernel.getProcedure(procedureName);
                assert(isNullAddress(retrievedAddress), "Procedure is deleted")

                await kernel.registerProcedure.call(procedureName, testAdder.address, capArray);
                await kernel.registerProcedure(procedureName, testAdder.address, capArray);
                const table3 = await kernel.returnRawProcedureTable.call()

                assert.deepEqual(table1, table3, 'Procedure Tables should be equal')

            })

            it('removing then registering a superset with same name', async function() {

                const kernel = await Kernel.new();
                const table_empty = await kernel.returnRawProcedureTable.call()

                const cap1 = new beakerlib.LogCap([]);
                const cap2 = new beakerlib.LogCap(["0xdeadbeef"]);
                const capArray1 = beakerlib.Cap.toInput([cap1]);
                const capArray2 = beakerlib.Cap.toInput([cap1, cap2]);

                const procedureName = "test";
                const testAdder = await testutils.deployedTrimmed(Valid.Adder);

                // First Create A Superset for initial state, so we have a referenc point
                await kernel.registerProcedure(procedureName, testAdder.address, capArray2);
                const table_sup = await kernel.returnRawProcedureTable.call()
                // Delete It
                await kernel.deleteProcedure(procedureName);
                const table_sup_del = await kernel.returnRawProcedureTable.call()
                assert.deepEqual(table_sup_del, table_empty, 'Procedure Tables should be equal after deletion')

                // Next Register a subset for initial state
                await kernel.registerProcedure(procedureName, testAdder.address, capArray1);
                await kernel.returnRawProcedureTable.call()
                // Delete It
                await kernel.deleteProcedure(procedureName);
                const table_sub_del = await kernel.returnRawProcedureTable.call()
                assert.deepEqual(table_sub_del, table_empty, 'Procedure Tables should be equal after deletion')

                 // Create A Superset - again
                 await kernel.registerProcedure(procedureName, testAdder.address, capArray2);
                 const table_sup_new = await kernel.returnRawProcedureTable.call()
                 // Check if matches the reference
                 assert.deepEqual(table_sup_new, table_sup, 'Procedure Tables should be equal')
            })
        })

        // TODO: this is not currently functional
        it.skip('should destroy the procedures contract on deletion', async function () {
            const kernel = await Kernel.new();
            const testAdder = await testutils.deployedTrimmed(Valid.Adder);
            const [err1, address] = await kernel.registerProcedure.call("test", testAdder.address, []);
            assert.equal(err1, 0);
            const tx1 = await kernel.registerProcedure('test', testAdder.address, []);
            const code = web3.eth.getCode(address);
            const codeAsNumber = web3.toBigNumber(code);
            // There should be some code at this address now
            // (here it is returned as a hex, we test the string because we
            // want to also be sure it is encoded as such)
            assert.notEqual(code, "0x0");

            const [err2, deleteAddress] = await kernel.deleteProcedure.call('test');
            assert.equal(err2, 0);

            const tx2 = await kernel.deleteProcedure('test');

            assert.equal(address, deleteAddress);
            const delcode = web3.eth.getCode(deleteAddress);
            const delcodeAsNumber = web3.toBigNumber(delcode);
            // If there is nothing at the address it returns the number zero
            // (here it is returned as a hex, we test the string because we
            // want to also be sure it is encoded as such)
            assert.equal(code, "0x0");
        })

        describe('should reject invalid key', function () {
            it('zero length (1)', async function () {
                const kernel = await Kernel.new();

                const [err, deleteAddress] = await kernel.deleteProcedure.call('');
                assert.equal(err, 1);
            });
        })
    })

    describe('.executeProcedure(bytes24 key, bytes payload)', function () {
        it('should return error if procedure key does not exist(3)', async function () {
            const kernel = await Kernel.new();

            const procName = "test";
            const functionSpec = "executeProcedure(bytes24,string,bytes)"
            const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
            const inputData = "0x" + functionSelectorHash
                + web3.fromAscii(procName.padEnd(24,"\0")).slice(2).padEnd(64,"0")
                + "60".padStart(64,"0")
                + "80".padStart(64,"0")

                + "00".padStart(64,"0")
                + "00".padStart(64,"0")
                ;
            const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
            assert.equal(valueXRaw, "0x03");
        });

        describe('should execute', function () {
            describe('Simple Procedure', function () {
                it('X() should fail', async function () {
                    // This now longer fails as we included a fallback function
                    const kernel = await Kernel.new();
                    const procName = "Simple";
                    const testSimple = await testutils.deployedTrimmed(Valid.Simple);
                    const [, address] = await kernel.registerAnyProcedure.call(procName, testSimple.address, []);
                    const tx = await kernel.registerAnyProcedure(procName, testSimple.address, []);

                    const functionSpec = "executeProcedure(bytes24,string,bytes)"
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);

                    const calledFunctionSpec = "X()";
                    const inputData = "0x" + functionSelectorHash
                        + web3.fromAscii(procName.padEnd(24,"\0")).slice(2).padEnd(64,"0")
                        + "60".padStart(64,"0")
                        + "a0".padStart(64,"0")

                        + "03".padStart(64,"0")
                        + web3.fromAscii("X()").slice(2).padEnd(64,"0")
                        + "00".padStart(64,"0")
                        ;
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});
                    // The error code is 0x55, because the procedure exists, it
                    // just doesn't have a function called "X()".
                    assert.equal(valueXRaw, "0x55");
                })

                it('A() should succeed', async function () {
                    const kernel = await Kernel.new();
                    const procName = "Simple";
                    const testSimple = await testutils.deployedTrimmed(Valid.Simple);
                    const [, address] = await kernel.registerAnyProcedure.call(procName, testSimple.address, []);
                    const tx = await kernel.registerAnyProcedure(procName, testSimple.address, []);

                    const functionSpec = "executeProcedure(bytes24,string,bytes)"
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);

                    const calledFunctionSpec = "A()";
                    const inputData = "0x" + functionSelectorHash
                        + web3.fromAscii(procName.padEnd(24,"\0")).slice(2).padEnd(64,"0")
                        + "60".padStart(64,"0")
                        + "a0".padStart(64,"0")

                        + "03".padStart(64,"0")
                        + web3.fromAscii(calledFunctionSpec).slice(2).padEnd(64,"0")
                        + "00".padStart(64,"0")
                        ;
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});

                    assert.equal(valueXRaw, "0x", "A() should succeed");
                })

                it('C() should fail without correctly specifying arguments', async function () {

                    const kernel = await Kernel.new();
                    const procName = "Simple";
                    const testSimple = await testutils.deployedTrimmed(Valid.Simple);
                    const [, address] = await kernel.registerAnyProcedure.call(procName, testSimple.address, []);
                    const tx = await kernel.registerAnyProcedure(procName, testSimple.address, []);

                    const functionSpec = "executeProcedure(bytes24,string,bytes)"
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);

                    const calledFunctionSpec = "C()";
                    const inputData = "0x" + functionSelectorHash
                        + web3.fromAscii(procName.padEnd(24,"\0")).slice(2).padEnd(64,"0")
                        + "60".padStart(64,"0")
                        + "a0".padStart(64,"0")

                        + "03".padStart(64,"0")
                        + web3.fromAscii(calledFunctionSpec).slice(2).padEnd(64,"0")
                        + "00".padStart(64,"0")
                        ;
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});

                    assert.equal(valueXRaw.slice(0,4), "0x55", "C() should not succeed");

                })

                it('C() should fail when using type synonyms such as uint, which cant be used in function selectors', async function () {

                    const kernel = await Kernel.new();
                    const procName = "Simple";
                    const testSimple = await testutils.deployedTrimmed(Valid.Simple);
                    const [, address] = await kernel.registerAnyProcedure.call(procName, testSimple.address, []);
                    const tx = await kernel.registerAnyProcedure(procName, testSimple.address, []);

                    const functionSpec = "executeProcedure(bytes24,string,bytes)"
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);

                    const calledFunctionSpec = "C(uint)";
                    const inputData = "0x" + functionSelectorHash
                        + web3.fromAscii(procName.padEnd(24,"\0")).slice(2).padEnd(64,"0")
                        + "60".padStart(64,"0")
                        + "a0".padStart(64,"0")

                        + "03".padStart(64,"0")
                        + web3.fromAscii(calledFunctionSpec).slice(2).padEnd(64,"0")
                        + "00".padStart(64,"0")
                        ;
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});

                    assert.equal(valueXRaw.slice(0,4), "0x55", "C(uint) should not succeed");
                })

                it('C(uint256) should succeed passing arguments', async function () {
                    const kernel = await Kernel.new();
                    const procName = "Simple";
                    const testSimple = await testutils.deployedTrimmed(Valid.Simple);
                    const [, address] = await kernel.registerAnyProcedure.call(procName, testSimple.address, []);
                    const tx = await kernel.registerAnyProcedure(procName, testSimple.address, []);

                    const functionSpec = "executeProcedure(bytes24,string,bytes)"
                    const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);

                    const calledFunctionSpec = "C(uint256)";
                    const inputData = "0x" + functionSelectorHash
                        + web3.fromAscii(procName.padEnd(24,"\0")).slice(2).padEnd(64,"0")
                        + "60".padStart(64,"0")
                        + "a0".padStart(64,"0")

                        + "03".padStart(64,"0")
                        + web3.fromAscii(calledFunctionSpec).slice(2).padEnd(64,"0")
                        + "00".padStart(64,"0")
                        ;
                    const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});

                    assert.equal(valueXRaw.slice(0,4), "0x55", "C(uint256) should not succeed");
                })
            })
        })

        describe('Discover Procedure Table', function () {
            it('should print a procedure table', async function () {
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8500,2);
                const cap2 = new beakerlib.WriteCap(0x8000,0);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                const SysCallTestWrite = await testutils.deployedTrimmed(Valid.SysCallTestWrite);
                const testSimple = await testutils.deployedTrimmed(Valid.Multiply);
                const tx1 = await kernel.registerProcedure("SysCallTestWrite", SysCallTestWrite.address, capArray);
                const tx2 = await kernel.registerProcedure("Simple", testSimple.address, []);
                // const rawProcTableData = await kernel.returnRawProcedureTable.call();
                const procTableData = await kernel.returnProcedureTable.call();

                // // Check that the two methods are the same
                // for (const v in procTableData) {
                //     console.log(v, ": " + web3.toHex(procTableData[v]));
                //     if (v > 24) break;
                // }

                const procTable = beakerlib.ProcedureTable.parse(procTableData);
                // console.log(beakerlib.ProcedureTable.stringify(procTable));
                let procedures = await kernel.listProcedures.call();
                assert.equal(procedures.length, Object.keys(procTable.procedures).length, "Same number of procedures as returned by listProcedures");
                for (let i = 0; i< procedures.length; i++) {
                    assert.equal(procedures[i], Object.keys(procTable.procedures)[i], "each procedure keys should be the same as returned by listProcedures");
                }

                const proc1 = procTable.procedures[procedures[0]];
                const proc2 = procTable.procedures[procedures[1]];

                assert.deepStrictEqual(
                    stripCapIndexVals(beakerlib.Cap.toCLists([cap1, cap2])),
                    stripCapIndexVals(proc1.caps),
                    "The requested caps should equal resulting caps");
            })
        })

        describe.skip('Entry Procedure', function () {
            it('should return the entry procedure address', async function () {
                const kernel = await Kernel.new();
                const procedureName = "Entry";
                const SysCallTestWrite = await testutils.deployedTrimmed(Valid.SysCallTestWrite);
                const [a, address] = await kernel.registerProcedure.call(procedureName, SysCallTestWrite.address, [3, 0x7, 0x80, 0x0]);
                // assert.equal(a.toNumber(), 0, "S() should succeed with zero errcode the second time");
                const tx = await kernel.registerProcedure(procedureName, SysCallTestWrite.address, [3, 0x7, 0x80, 0x0]);
                const valueA = await kernel.getProcedure.call(procedureName);
                // const
                // console.log(errA, valueA);
                // console.log(errA)
                // console.log("valueA:", valueA)

                // // need to have the ABI definition in JSON as per specification
                // const valueX = await kernel.executeProcedure.call("SysCallTestWrite", "S()", "");
                // await kernel.executeProcedure("SysCallTestWrite", "S()", "");
                // assert.equal(valueX.toNumber(), 4, "S() should succeed with correct value the first time");

                // // do it again
                // const [err2, value2] = await kernel.executeProcedure.call("SysCallTestWrite", "S()", "");
                // await kernel.executeProcedure("SysCallTestWrite", "S()", "");
                // assert.equal(err2.toNumber(), 0, "S() should succeed with zero errcode the second time");
                // assert.equal(value2.toNumber(), 5, "S() should succeed with correct value the second time");
            })
        })

        it('should return an error if key does not exist (3)', async function () {
            const kernel = await Kernel.new();

            const procName = "test";
            const functionSpec = "executeProcedure(bytes24,string,bytes)"
            const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);

            const calledFunctionSpec = "";
            const inputData = "0x" + functionSelectorHash
                + web3.fromAscii(procName.padEnd(24,"\0")).slice(2).padEnd(64,"0")
                + "60".padStart(64,"0")
                + "80".padStart(64,"0")

                + "00".padStart(64,"0")
                + "00".padStart(64,"0")
                ;
            const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});

            assert.equal(valueXRaw.slice(0,4), "0x03", "non-existent procedure should fail, with 0x03");

        });

        describe('should return an error if procedure return error when', function () {
            it('receives invalid arguments')
            it('throws an error', async function () {
                let kernel = await Kernel.new();

                const testDivide = await testutils.deployedTrimmed(Valid.Divide);
                let [err, address] = await kernel.registerProcedure.call('TestDivide', testDivide.address, []);
                let tx1 = await kernel.registerProcedure('TestDivide', testDivide.address, []);

                assert(web3.isAddress(address), `Procedure Address (${address}) is a real address`)
                assert(!isNullAddress(address), 'Procedure Address is not null')

                let divide = Valid.Divide.at(address);
                let result = await divide.divide.call(8, 2);
                assert.equal(result.toNumber(), 4);

                // The returned code should be the same as the sent code
                const code = web3.eth.getCode(address);
                assert.equal(testutils.trimSwarm(Valid.Divide.deployedBytecode), code);

                // Try dividing by zero
                try {
                    const divideByZero = await divide.divide.call(8, 0);
                } catch (e) {
                    assert.equal(e, "Error: VM Exception while processing transaction: invalid opcode");
                }
            });
        })

        describe('should reject invalid key', function () {
            it('zero length (1)', async function () {
                const kernel = await Kernel.new();

                const procName = "";
                const functionSpec = "executeProcedure(bytes24,string,bytes)"
                const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);

                const calledFunctionSpec = "";
                const inputData = "0x" + functionSelectorHash
                    + web3.fromAscii(procName.padEnd(24,"\0")).slice(2).padEnd(64,"0")
                    + "60".padStart(64,"0")
                    + "80".padStart(64,"0")

                    + "00".padStart(64,"0")
                    + "00".padStart(64,"0")
                    ;
                const valueXRaw = await web3.eth.call({to: kernel.address, data: inputData});

                assert.equal(valueXRaw.slice(0,4), "0x01", "non-existent procedure should fail, with 0x01");
            });
        })
    })
})

// Test hack to remove data we don't care about. The kernel stores no
// information about where a capability was derived from.
function stripCapIndexVals(capData) {
    for (const cap in capData) {
        cap.capIndex = 0;
    }
}
