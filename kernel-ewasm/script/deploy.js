const Web3 = require("web3");
const fs = require("fs");
const path = require("path")

// Get BuildPath
const buildPath = path.resolve(process.env.PWD, './build')

async function main() {
    // Connect to our local node
    const web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8545"));

    // NOTE: if you run Kovan node there should be an address you've got in the "Option 2: Run Kovan node" step
    
    const accounts = await web3.eth.personal.getAccounts()
    web3.eth.defaultAccount = web3.utils.toChecksumAddress(accounts[0])
    
    console.log(accounts)

    // read JSON ABI
    const abi = JSON.parse(fs.readFileSync(path.resolve(buildPath, "./TokenContract.json")));
    // convert Wasm binary to hex format
    const codeHex = '0x' + fs.readFileSync(path.resolve(buildPath, "./kernel-ewasm.wasm")).toString('hex');

    const TokenContract = new web3.eth.Contract(abi, { data: codeHex, from: web3.utils.toChecksumAddress(accounts[0]) });

    const TokenDeployTransaction = TokenContract.deploy({ data: codeHex, arguments: [web3.utils.numberToHex(10000000)] });
    
    await web3.eth.personal.unlockAccount(accounts[0], "", null)
    let gas = await TokenDeployTransaction.estimateGas()
    
    let contract = await TokenDeployTransaction.send({ gasLimit: gas, from: accounts[0] })
    console.log("Address of new contract: " + contract.options.address);

    // Check balance of recipient. Should print 200
    let rec_balance = await contract.methods.balanceOf(accounts[0]).call();
    console.log(`Receipient Balance: ${rec_balance}`)

    // Check balance of sender (owner of the contract). Should print 10000000 - 200 = 9999800
    let owner_balance = await contract.methods.balanceOf(web3.eth.defaultAccount).call();
    console.log(`Owner Balance: ${owner_balance}`)
}

main()