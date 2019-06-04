const Web3 = require("web3");
const fs = require("fs");
const path = require("path")
const http = require('http')
const assert = require('assert')

// Get BuildPath
const BUILD_PATH = path.resolve(process.cwd(), './build')

// Get Dev Chain Config
const CHAIN_CONFIG = require(path.resolve(process.cwd(), './wasm-dev-chain.json'));

// Web3 Config
const WEB3_OPTIONS = {
    transactionConfirmationBlocks: 1
}

const DEFAULT_ACCOUNT = {
    NAME: 'user',
    PASSWORD: 'user'
}

const port = 8545;
// Connect to our local node
const web3 = new Web3(new Web3.providers.HttpProvider(`http://localhost:${port}`), null, WEB3_OPTIONS);


// Create Account
function createAccount(name, password) {
    var headers = {
        'Content-Type': 'application/json'
    };

    var dataString = JSON.stringify({ "jsonrpc": "2.0", "method": "parity_newAccountFromPhrase", "params": [name, password], "id": 0 });

    var options = {
        hostname: '127.0.0.1',
        port: port,
        method: 'POST',
        headers: headers
    };

    return new Promise((resolve, reject) => {
        const req = http.request(options, res => {
            res.setEncoding('utf8')
            let chunk = ''
            res.on('data', data => {
                // console.log(data);
                chunk += data;
            })
            res.on('end', () => {
                resolve(chunk)
            })
            res.on('error', reject)
        })

        req.on('error', (e) => {
            reject(`Problem with request: ${e.message}`)
        });

        req.write(dataString)
        req.end();
    })
}

async function newKernelInstance() {
       // Create Account
       const newAccount = await createAccount(DEFAULT_ACCOUNT.NAME, DEFAULT_ACCOUNT.PASSWORD);
    //    console.log(`Created account: ${newAccount}`)

    //    console.log(`Fetching addresss`)
       const accounts = await web3.eth.personal.getAccounts()
    //    console.log(`Got ${accounts.length} accounts`)
       if (accounts.length == 0) throw `Got zero accounts`;

       const account = web3.utils.toChecksumAddress(accounts[0], web3.utils.hexToNumber(CHAIN_CONFIG.params.networkId));
    //    console.log(`Set Account: ${account}`)

       web3.eth.defaultAccount = account;

       // read JSON ABI
       const abi = JSON.parse(fs.readFileSync(path.resolve(BUILD_PATH, "./TokenInterface.json")));
       // convert Wasm binary to hex format
       const codeHex = '0x' + fs.readFileSync(path.resolve(BUILD_PATH, "./kernel-ewasm.wasm")).toString('hex');

       const TokenContract = new web3.eth.Contract(abi, null, { data: codeHex, from: account, transactionConfirmationBlocks: 1 });
       const TokenDeployTransaction = TokenContract.deploy({ data: codeHex, arguments: [355] });

       await web3.eth.personal.unlockAccount(accounts[0], "user", null)
    //    console.log(`Unlocked Account: ${accounts[0]}`);

       let gas = await TokenDeployTransaction.estimateGas()
    //    console.log(`Estimated Gas Cost: ${gas}`)

       let contract_tx = TokenDeployTransaction.send({ gasLimit: gas, from: account })

       let tx_hash = await new Promise((res, rej) => contract_tx.on('transactionHash', res).on('error', rej));

       let tx_receipt = await web3.eth.getTransactionReceipt(tx_hash);
       let contract_addr = tx_receipt.contractAddress;

    //    console.log("Address of new contract: " + contract_addr);

       let contract = TokenContract.clone();
       contract.address = contract_addr;

       return contract;

}

describe('Kernel', function() {

    describe('constructor', function() {
        this.timeout(20000);
        it('should have initial balance', async function() {
            let contract = await newKernelInstance();
            const accounts = await web3.eth.personal.getAccounts()
            // Check balance of recipient. Should print 355
            let rec_balance = await contract.methods.balanceOf(accounts[0]).call();
            assert.strictEqual(web3.utils.hexToNumber(rec_balance), 355)

             // Check balance of sender (owner of the contract). Should print 10000000 - 355 = 9999645
            // let owner_balance = await contract.methods.balanceOf(web3.eth.defaultAccount).call();
            // assert.strictEqual(web3.utils.hexToNumber(owner_balance), 9999800)

        })
    })

    describe('validator', function() {
        this.timeout(20000);
        let kernel;
        before(async function () {
            kernel = await newKernelInstance();

        })
        it('should return false when given the null address', async function() {
        this.timeout(20000);
        let rec_validation = await kernel.methods.check_contract('0x0000000000000000000000000000000000000000').call();
            assert.strictEqual(rec_validation, false)
        })
        it('should return true when given a valid address', async function() {
            const accounts = await web3.eth.personal.getAccounts()
            assert(web3.utils.isAddress(accounts[0]), "The example should be a valid address")
            let rec_validation = await kernel.methods.check_contract(accounts[0]).call();
            assert.strictEqual(rec_validation, true)
        })
        it('should return the code size of the kernel', async function() {
            const kernelAddress = kernel.options.address;
            assert(web3.utils.isAddress(kernelAddress), "The kernel address should be a valid address")
            let rec_validation = await kernel.methods.get_code_size(kernelAddress).call();
            assert.strictEqual(typeof rec_validation, "number")
        })
        // it('should return false when trying to validate the kernel itself', async function() {
        //     const kernelAddress = kernel.options.address;
        //     assert(web3.utils.isAddress(kernelAddress), "The kernel address should be a valid address")
        //     let rec_validation = await kernel.methods.check_contract(kernelAddress).call();
        //     assert.strictEqual(rec_validation, false)
        // })

        it('should copy the code of the kernel', async function() {
            const kernelAddress = kernel.options.address;
            assert(web3.utils.isAddress(kernelAddress), "The kernel address should be a valid address")
            const code_size = await kernel.methods.get_code_size(kernelAddress).call();
            const code_hex = await kernel.methods.code_copy(kernelAddress).call();
            const code = web3.utils.hexToBytes(code_hex);
            assert.strictEqual(code_size, code.length, "The code length should be as given by EXTCODESIZE");
        })
    })
})
