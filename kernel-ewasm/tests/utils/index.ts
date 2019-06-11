import { Contract } from "web3-eth-contract";
import * as utils from 'web3-utils';

const BN = require('bn.js')
const Web3 = require('web3');
const fs = require("fs");
const path = require("path")
const http = require('http')

type BN = typeof BN;

// Get BuildPath
const TARGET_PATH = path.resolve(process.cwd(), './target');
// Get Dev Chain Config
const CHAIN_CONFIG = require(path.resolve(process.cwd(), './wasm-dev-chain.json'));
// Web3 Config
const WEB3_OPTIONS = {
    transactionConfirmationBlocks: 1
};

// Kernel Code
const KERNEL_WASM = '0x' + fs.readFileSync(path.resolve(TARGET_PATH, "./cap9-kernel.wasm")).toString('hex');
const KERNEL_WASM_ABI = JSON.parse(fs.readFileSync(path.resolve(TARGET_PATH, "./json/KernelInterface.json")))

const DEFAULT_ACCOUNT = {
    NAME: 'user',
    PASSWORD: 'user'
};
const DEFAULT_PORT = 8545;

// Connect to our local node
export const web3 = new Web3(new Web3.providers.HttpProvider(`http://localhost:${DEFAULT_PORT}`), null, WEB3_OPTIONS);

export class KernelInstance {

    constructor(public contract: Contract) { }

    async getEntryProcedure(): Promise<string> {
        return web3.utils.hexToAscii(await this.contract.methods.entryProcedure().call()).replace(/\0.*$/g, '');
    }

    async getCurrentProcedure(): Promise<string> {
        return web3.utils.hexToAscii(await this.contract.methods.currentProcedure().call()).replace(/\0.*$/g, '');
    }

    async getProcCapTypeLen(proc_key: string, cap_type: CAP_TYPE): Promise<number> {
        return utils.toDecimal(await this.contract.methods.get_cap_type_len(proc_key, cap_type).call());
    }

}

export enum CAP_TYPE {
    PROC_CALL = 3,
    PROC_REGISTER = 4,
    PROC_DELETE = 5,
    PROC_ENTRY = 6,
    STORE_WRITE = 7,
    LOG = 8,
    ACC_CALL = 9
}

export interface Capability {
    cap_type: CAP_TYPE;
    to_input(): Array<number | string>;
}

export class NewCap {
    constructor(public parent_index: number, public cap: Capability) {}
    to_input(): (string| number)[] {
        let cap_input = this.cap.to_input();
        let cap_size = cap_input.length + 3;
        return [cap_size, this.cap.cap_type, this.parent_index].concat(cap_input as any) as any
    }
}

export class CallCap implements Capability {
    public cap_type = CAP_TYPE.PROC_CALL;
    constructor(public prefixLength: number, public baseKey: string) { }
    to_input(): number[] {
        // The baseKey will take up the last 24 bytes
        // baseKey24 is the given key correctly padded to 24 bytes, left aligned
        const baseKey24 = utils.fromAscii(this.baseKey.padEnd(24, '\0'))
        // baseKeyHex is baseKey24, hex-encoded, and is therefore 48 chars. The
        // "0x" is removed from the start of the string.
        const baseKeyHex = utils.toHex(baseKey24).slice(2);
        // prefixHex is the prefix length hex-encoded and padded to two chars (a
        // single byte). The "0x" is removed here also.
        const prefixHex = utils.toHex(this.prefixLength).slice(2).padStart(2,'0');
        // There are 7 bytes between the prefix length and the start of the base
        // key.
        const undefinedFill = utils.toHex("".padEnd(7,'\0')).slice(2);
        // We string these together in the correct order.
        const key = "0x" + prefixHex + undefinedFill + baseKeyHex;

        return [key as any]
    }
}

export class RegisterCap implements Capability {
    public cap_type = CAP_TYPE.PROC_REGISTER;
    constructor(public prefixLength: number, public baseKey: string) { }
    to_input(): number[] {
        // The baseKey will take up the last 24 bytes
        // baseKey24 is the given key correctly padded to 24 bytes, left aligned
        const baseKey24 = utils.fromAscii(this.baseKey.padEnd(24, '\0'))
        // baseKeyHex is baseKey24, hex-encoded, and is therefore 48 chars. The
        // "0x" is removed from the start of the string.
        const baseKeyHex = utils.toHex(baseKey24).slice(2);
        // prefixHex is the prefix length hex-encoded and padded to two chars (a
        // single byte). The "0x" is removed here also.
        const prefixHex = utils.toHex(this.prefixLength).slice(2).padStart(2,'0');
        // There are 7 bytes between the prefix length and the start of the base
        // key.
        const undefinedFill = utils.toHex("".padEnd(7,'\0')).slice(2);
        // We string these together in the correct order.
        const key = "0x" + prefixHex + undefinedFill + baseKeyHex;

        return [key as any]
    }
}

export class DeleteCap implements Capability {
    public cap_type = CAP_TYPE.PROC_DELETE;
    constructor(public prefixLength: number, public baseKey: string) { }
    to_input(): number[] {
        // The baseKey will take up the last 24 bytes
        // baseKey24 is the given key correctly padded to 24 bytes, left aligned
        const baseKey24 = utils.fromAscii(this.baseKey.padEnd(24, '\0'))
        // baseKeyHex is baseKey24, hex-encoded, and is therefore 48 chars. The
        // "0x" is removed from the start of the string.
        const baseKeyHex = utils.toHex(baseKey24).slice(2);
        // prefixHex is the prefix length hex-encoded and padded to two chars (a
        // single byte). The "0x" is removed here also.
        const prefixHex = utils.toHex(this.prefixLength).slice(2).padStart(2,'0');
        // There are 7 bytes between the prefix length and the start of the base
        // key.
        const undefinedFill = utils.toHex("".padEnd(7,'\0')).slice(2);
        // We string these together in the correct order.
        const key = "0x" + prefixHex + undefinedFill + baseKeyHex;

        return [key as any]
    }
}

export class EntryCap implements Capability {
    public cap_type = CAP_TYPE.PROC_ENTRY;
    to_input(): number[] {
        return []
    }
}

export class WriteCap implements Capability {
    public cap_type = CAP_TYPE.STORE_WRITE;
    constructor(public location: number, public size: number) { }
    to_input(): number[] {
        return [this.location, this.size]
    }
}

export class LogCap implements Capability {
    public cap_type = CAP_TYPE.LOG;
    constructor(public topics: string[]) { if (topics.length > 4) throw "Too many topics"; }
    to_input(): any[] {
        const topic1 = (this.topics.length >= 1) ? this.topics[0] : 0;
        const topic2 = (this.topics.length >= 2) ? this.topics[1] : 0;
        const topic3 = (this.topics.length >= 3) ? this.topics[2] : 0;
        const topic4 = (this.topics.length >= 4) ? this.topics[3] : 0;
        return [this.topics.length].concat([topic1, topic2, topic3, topic4].map(utils.utf8ToHex) as any)
    }
}

export class AccCallCap implements Capability {
    public cap_type = CAP_TYPE.ACC_CALL;
    constructor(public callAny: boolean, public sendValue: boolean, public address: string) {};
    to_input(): string[] {
        const value = new Uint8Array(32);
        const callAny = this.callAny ? 0b10000000 : 0;
        const sendValue = this.sendValue ? 0b01000000 : 0;
        value[0] = callAny | sendValue;
        if (!this.address) {
            value.fill(0,12,32);
        } else {
            const byteArray = utils.hexToBytes(this.address);
            value.set(byteArray, 32 - byteArray.length);
        }

        return [utils.bytesToHex(value as any) as any];
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

export async function deployContract(file_name: string, abi_name: string): Promise<Contract> {
    // Create Account
    const newAccount = await createAccount(DEFAULT_ACCOUNT.NAME, DEFAULT_ACCOUNT.PASSWORD);
    const accounts = await web3.eth.personal.getAccounts();
    if (accounts.length == 0)
        throw `Got zero accounts`;

    const account = web3.utils.toChecksumAddress(accounts[0], web3.utils.hexToNumber(CHAIN_CONFIG.params.networkId));
    web3.eth.defaultAccount = account;

    // read JSON ABI
    const abi = JSON.parse(fs.readFileSync(path.resolve(TARGET_PATH, `./json/${abi_name}.json`)));

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

export async function newKernelInstance(proc_key: string, proc_address: string, cap_list: NewCap[] = []): Promise<KernelInstance> {
    // Create Account
    const newAccount = await createAccount(DEFAULT_ACCOUNT.NAME, DEFAULT_ACCOUNT.PASSWORD);
    const accounts = await web3.eth.personal.getAccounts();
    if (accounts.length == 0)
        throw `Got zero accounts`;
    const account = web3.utils.toChecksumAddress(accounts[0], web3.utils.hexToNumber(CHAIN_CONFIG.params.networkId));
    web3.eth.defaultAccount = account;

    const abi = KERNEL_WASM_ABI
    const codeHex = KERNEL_WASM

    // Encode CapList
    let encoded_cap_list: string[] = cap_list.reduce((payload, cap) => payload.concat(cap.to_input()), []);

    const KernelContract = new web3.eth.Contract(abi, null, { data: codeHex, from: account, transactionConfirmationBlocks: 1 } as any);
    const TokenDeployTransaction = KernelContract.deploy({ data: codeHex, arguments: [proc_key, proc_address, encoded_cap_list] });
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
