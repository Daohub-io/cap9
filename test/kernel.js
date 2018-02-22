const debug = require('debug')
const assert = require('assert')

const Kernel = artifacts.require('./Kernel.sol')
const Dog = artifacts.require('./Dog.sol');
const testDebug = debug('test:kernel')

contract('Kernel', function (accounts) {

    it('should sest owner to msg.sender', async function () {     
        let kernel = await Kernel.deployed();
        let owner = await kernel.owner.call()  
        return assert.equal(owner, accounts[0]);
    })

    it('should create a dog if given dog', async function () {
        let kernel = await Kernel.deployed();
        let _ = await kernel.create(Dog.bytecode, {from: accounts[0]});
        
        let dog_address = await kernel.dog.call();
        let dog = Dog.at(dog_address)

        let name = await dog.name.call({from: accounts[0]})
        console.log(name)
        assert.equal(name, "happy");

    })

})
