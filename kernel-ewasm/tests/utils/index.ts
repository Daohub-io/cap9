import { Contract } from "web3-eth-contract";

const Web3 = require('web3')
const fs = require("fs");
const path = require("path")
const http = require('http')

// Get BuildPath
const TARGET_PATH = path.resolve(process.cwd(), './target');
// Get Dev Chain Config
const CHAIN_CONFIG = require(path.resolve(process.cwd(), './wasm-dev-chain.json'));
// Web3 Config
const WEB3_OPTIONS = {
    transactionConfirmationBlocks: 1
};

// Kernel Code
const KERNEL_WASM =  '0x' + fs.readFileSync(path.resolve(TARGET_PATH, "./cap9-kernel.wasm")).toString('hex');
const KERNEL_WASM_ABI =  JSON.parse(fs.readFileSync(path.resolve(TARGET_PATH, "./json/KernelInterface.json")))

const DEFAULT_ACCOUNT = {
    NAME: 'user',
    PASSWORD: 'user'
};
const DEFAULT_PORT = 8545;

// Connect to our local node
export const web3 = new Web3(new Web3.providers.HttpProvider(`http://localhost:${DEFAULT_PORT}`), null, WEB3_OPTIONS);

export class KernelInstance {

    constructor(public contract: Contract) {}

    async getEntryProcedure(): Promise<string> {
        return web3.utils.hexToAscii(await this.contract.methods.entryProcedure().call()).replace(/\0.*$/g,'');
    }

    async getCurrentProcedure(): Promise<string> {
        return web3.utils.hexToAscii(await this.contract.methods.currentProcedure().call()).replace(/\0.*$/g,'');
    }

}

// Create Account
export function createAccount(name, password): Promise<string> {
    var headers = {
        'Content-Type': 'application/json'
    };
    var dataString = JSON.stringify({ "jsonrpc": "2.0", "method": "parity_newAccountFromPhrase", "params": [name, password], "id": 0 });
    var options = {
        hostname: '127.0.0.1',
        port: DEFAULT_PORT,
        method: 'POST',
        headers: headers
    };
    return new Promise((resolve, reject) => {
        const req = http.request(options, res => {
            res.setEncoding('utf8');
            let chunk = '';
            res.on('data', data => {
                // console.log(data);
                chunk += data;
            });
            res.on('end', () => {
                resolve(chunk);
            });
            res.on('error', reject);
        });
        req.on('error', (e) => {
            reject(`Problem with request: ${e.message}`);
        });
        req.write(dataString);
        req.end();
    });
}

export async function newTestContract(file_name: string, abi_name: string): Promise<Contract> {
    // Create Account
    const newAccount = await createAccount(DEFAULT_ACCOUNT.NAME, DEFAULT_ACCOUNT.PASSWORD);
    const accounts = await web3.eth.personal.getAccounts();
    if (accounts.length == 0)
        throw `Got zero accounts`;

    const account = web3.utils.toChecksumAddress(accounts[0], web3.utils.hexToNumber(CHAIN_CONFIG.params.networkId));
    web3.eth.defaultAccount = account;

    // read JSON ABI
    const abi = JSON.parse(fs.readFileSync(path.resolve(TARGET_PATH,`./json/${abi_name}.json`)));

    // convert Wasm binary to hex format
    const codeHex = '0x' + fs.readFileSync(path.resolve(TARGET_PATH, `./${file_name}.wasm`)).toString('hex');
    const Contract = new web3.eth.Contract(abi, null, { data: codeHex, from: account, transactionConfirmationBlocks: 1 } as any);
    const DeploymentTx = Contract.deploy({ data: codeHex });

    await web3.eth.personal.unlockAccount(accounts[0], "user", null);
    let gas = await DeploymentTx.estimateGas();
    let contract_tx = DeploymentTx.send({ gasLimit: gas, from: account } as any);
    let tx_hash: string = await new Promise((res, rej) => contract_tx.on('transactionHash', res).on('error', rej));
    let tx_receipt = await web3.eth.getTransactionReceipt(tx_hash);
    let contract_addr = tx_receipt.contractAddress;
    let contract = Contract.clone();
    contract.address = contract_addr;

    return contract;
}

export async function newKernelInstance(proc_key: string, proc_address: string): Promise<KernelInstance> {
    // Create Account
    const newAccount = await createAccount(DEFAULT_ACCOUNT.NAME, DEFAULT_ACCOUNT.PASSWORD);
    const accounts = await web3.eth.personal.getAccounts();
    if (accounts.length == 0)
        throw `Got zero accounts`;
    const account = web3.utils.toChecksumAddress(accounts[0], web3.utils.hexToNumber(CHAIN_CONFIG.params.networkId));
    web3.eth.defaultAccount = account;

    const abi = KERNEL_WASM_ABI
    const codeHex = KERNEL_WASM

    const KernelContract = new web3.eth.Contract(abi, null, { data: codeHex, from: account, transactionConfirmationBlocks: 1 } as any);
    const TokenDeployTransaction = KernelContract.deploy({ data: codeHex, arguments: [proc_key, proc_address] });
    await web3.eth.personal.unlockAccount(accounts[0], "user", null);
    let gas = await TokenDeployTransaction.estimateGas();
    let contract_tx = TokenDeployTransaction.send({ gasLimit: gas, from: account } as any);
    let tx_hash: string = await new Promise((res, rej) => contract_tx.on('transactionHash', res).on('error', rej));
    let tx_receipt = await web3.eth.getTransactionReceipt(tx_hash);
    let contract_addr = tx_receipt.contractAddress;
    let contract = KernelContract.clone();
    contract.address = contract_addr;
    return new KernelInstance(contract);
}
