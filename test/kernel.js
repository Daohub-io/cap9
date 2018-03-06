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
let testAccount = 0;

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
        it('should create valid procedure')
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
    describe('.executeProcedure()')
}