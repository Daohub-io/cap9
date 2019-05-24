const Web3 = require("web3");
const fs = require("fs");
const path = require("path")
const http = require('http')

// Get BuildPath
const BUILD_PATH = path.resolve(process.env.PWD, './build')

// Get Dev Chain Config
const CHAIN_CONFIG = require(path.resolve(process.env.PWD, './wasm-dev-chain.json'));

// Web3 Config
const WEB3_OPTIONS = {
    transactionConfirmationBlocks: 1
}

const DEFAULT_ACCOUNT = {
    NAME: 'user',
    PASSWORD: 'user'
}

// Create Account
function createAccount(name, password) {
    var headers = {
        'Content-Type': 'application/json'
    };

    var dataString = JSON.stringify({ "jsonrpc": "2.0", "method": "parity_newAccountFromPhrase", "params": [name, password], "id": 0 });

    var options = {
        hostname: '127.0.0.1',
        port: 8546,
        method: 'POST',
        headers: headers
    };

    return new Promise((resolve, reject) => {
        const req = http.request(options, res => {
            console.log(`STATUS: ${res.statusCode}`);
            console.log(`HEADERS: ${JSON.stringify(res.headers)}`);
            res.setEncoding('utf8')
            let chunk = ''
            res.on('data', data => {
                console.log(data);
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

async function main() {

    // Connect to our local node
    const web3 = new Web3("ws://localhost:8546", null, WEB3_OPTIONS);

    // Create Account
    const newAccount = await createAccount(DEFAULT_ACCOUNT.NAME, DEFAULT_ACCOUNT.PASSWORD);
    console.log(`Created account: ${newAccount}`)

    console.log(`Fetching addresss`)
    const accounts = await web3.eth.personal.getAccounts()
    console.log(`Got ${accounts.length} accounts`)
    if (accounts.length == 0) throw `Got zero accounts`;

    const account = web3.utils.toChecksumAddress(accounts[0], web3.utils.hexToNumber(CHAIN_CONFIG.params.networkId));
    console.log(`Set Account: ${account}`)

    web3.eth.defaultAccount = account;

    // read JSON ABI
    const abi = JSON.parse(fs.readFileSync(path.resolve(BUILD_PATH, "./TokenInterface.json")));
    // convert Wasm binary to hex format
    const codeHex = '0x' + fs.readFileSync(path.resolve(BUILD_PATH, "./kernel-ewasm.wasm")).toString('hex');

    const TokenContract = new web3.eth.Contract(abi, null, { data: codeHex, from: account, transactionConfirmationBlocks: 1 });
    const TokenDeployTransaction = TokenContract.deploy({ data: codeHex, arguments: [100] });

    await web3.eth.personal.unlockAccount(accounts[0], "user", null)
    console.log(`Unlocked Account: ${accounts[0]}`);

    let gas = await TokenDeployTransaction.estimateGas()
    console.log(`Estimated Gas Cost: ${gas}`)

    let contract_tx = TokenDeployTransaction.send({ gasLimit: gas, from: account })

    let tx_hash = await new Promise((res, rej) => contract_tx.on('transactionHash', res).on('error', rej));

    let tx_receipt = await web3.eth.getTransactionReceipt(tx_hash);
    let contract_addr = tx_receipt.contractAddress;

    console.log("Address of new contract: " + contract_addr);

    let contract = TokenContract.clone();
    contract.address = contract_addr;

    // Check balance of recipient. Should print 200
    let rec_balance = await contract.methods.balanceOf(accounts[0]).call();
    console.log(`Receipient Balance: ${rec_balance}`)

    // Check balance of sender (owner of the contract). Should print 10000000 - 200 = 9999800
    let owner_balance = await contract.methods.balanceOf(web3.eth.defaultAccount).call();
    console.log(`Owner Balance: ${owner_balance}`)
}

main()