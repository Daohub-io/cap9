const debug = require('debug')
const assert = require('assert')

const Factory = artifacts.require('./Factory.sol')
const Adder = artifacts.require('./Adder.sol');
const testDebug = debug('test:Factory')

contract('Factory', function (accounts) {

    it('should create a adder contract', async function () {
        let factory = await Factory.deployed();
        let address = await factory.create(Adder.bytecode, {from: accounts[0]});

        let adder = Adder.at(address)
        let two = await Adder.add.call(1, 1);
        assert.equal(two, 2);
    })

})
