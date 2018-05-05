
const fs = require("fs")
const path = require("path")
const assert = require("assert")
const Web3 = require("web3")

// Connect to our local node
const web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8545"))

async function Kernel(...arguments) {
    web3.eth.defaultAccount = await web3.eth.getAccounts().then(a => a[0])
    // read JSON ABI
    const abi = JSON.parse(fs.readFileSync(path.resolve(__dirname, "../target/json/KernelContract.json")))
    // convert Wasm binary to hex format
    const codeHex = '0x' + fs.readFileSync(path.resolve(__dirname, "../target/beaker_core.wasm")).toString('hex')

    const KernelContract = new web3.eth.Contract(abi, { data: codeHex, from: web3.eth.defaultAccount })

    // Unlock Accounts for Parity
    await web3.eth.personal.unlockAccount(web3.eth.defaultAccount, "user")

    // Creates and Deploys a new Kernel Instance
    const KernelDeployTransaction = KernelContract.deploy({ data: codeHex, arguments })
    const gas = await KernelDeployTransaction.estimateGas()

    return KernelDeployTransaction.send({ gasLimit: gas, from: web3.eth.defaultAccount })
}

describe('Kernel', function () {

    // Add a 2 second time buffer for each deployment
    afterEach(function (done) {
        this.timeout(3000)
        setTimeout(done, 2000)
    })

    describe('deploy', function () {

        it('should have valid address', async function () {
            let kernel = await Kernel(0)
            let address = kernel.options.address
            assert(web3.utils.isHex(address))
        })

    })

    describe('call', function () {

        describe('.version', function () {

            it('should return valid version', async function () {
                let kernel = await Kernel(1234)
                let version = await kernel.methods.version().call()
                version = web3.utils.hexToNumber(version)
                assert.equal(version, 1234)
            })
        })
    })

})