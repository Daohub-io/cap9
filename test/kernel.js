const debug = require('debug')
const assert = require('assert')

const Kernel = artifacts.require('./Kernel.sol')


// Valid Contracts
const Valid =  {
    Adder: artifacts.require('test/valid/Adder.sol')
}

// Test utility functions
function isNullAddress(address) {
    return address === "0x0000000000000000000000000000000000000000";
}

const testDebug = debug('test:Factory')
const testAccount = 0;

contract('Kernel', function (accounts) {

    describe('.listProcedures()', function () {
        it('should return nothing if zero procedures')
        it('should return existing procedure keys')
        it('should return a list of procedures which can be retrieved')
    })
    describe('.getProcedure()', function () {
        it('should return a non-zero address iff procedure exists')
        it('should return a zero address iff procedure does not exist')
    })

    describe('.createProcedure()', function() {
        it('should create valid procedure', async function () {
            let kernel = await Kernel.new();

            let address = await kernel.createProcedure.call('TestAdder', Valid.Adder.bytecode)
            let tx1 = await kernel.createProcedure('TestAdder', Valid.Adder.bytecode)

            assert(web3.isAddress(address), `Procedure Address (${address}) is a real address`)
            assert(!isNullAddress(address), 'Procedure Address is not null')

            let adder = Valid.Adder.at(address);
            assert.equal(await adder.add.call(1, 1), 2)
        })

        it('should reject invalid payload')

        describe('should reject invalid key', function () {
            it('excess length')
            it('zero length')
            it('duplicate procedure key')
        })
    })

    describe('.deleteProcedure()', function () {
        it('should return error if procedure key does not exist')
        it('should return deleted procedure address if procedure key is valid ')

        // On deletion, kernel should destroy contract instance
        it.skip('should destroy the procedures contract on deletion', async function () {
            let kernel = new Kernel.new();

            let address = await kernel.createProcedure.call("test", Adder.bytecode);
            let tx1 = await kernel.createProcedure('test', Adder.bytecode)

            let delete_address = await kernel.deleteProcedure.call('test');
            let tx2 = await kernel.deleteProcedure('test');

            // How do we test if deleted address is destroyed???

        })

        describe('should reject invalid key', function () {
            it('excess length')
            it('zero length')
        })
    })

    describe('.executeProcedure(bytes32 key, bytes payload)', function () {

        describe('should return a valid value for', function () {
            it.skip('Adder Procedure', async function () {
                const kernel = await Kernel.new();
                let address = await kernel.createProcedure.call("TestAdder", Adder.bytecode, {from: accounts[testAccount]});
                let tx = await kernel.createProcedure("TestAdder", Adder.bytecode, {from: accounts[testAccount]});
                assert(web3.isAddress(address), `The returned address (${address}) is a valid address`);
                assert(!isNullAddress(address), `The returned address (${address}) is not the null address`);

                let tl = await kernel.executeProcedure("TestAdder", {from: accounts[testAccount]});
                let tc = await kernel.executeProcedure.call("TestAdder", {from: accounts[testAccount]});
                console.log(tc.toNumber());
                let adder = Adder.at(address);
                let two = await adder.add.call(1, 1);
                assert.equal(two, 2);
                console.log("result: ",  two)

                // The returned code should be the same as the sent code
                const code = web3.eth.getCode(address);
                assert.equal(Adder.deployedBytecode, code);
            })
        })

        it('should return an error if key does not exist')

        describe('should return an error if procedure return error when', function () {
            it('recieves invalid arguments')
            it('throws an error')
        })


    })
})