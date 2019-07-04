import { Contract } from "web3-eth-contract";
import * as utils from 'web3-utils';
const assert = require('assert')

const BN = require('bn.js')
const Web3 = require('web3');
const fs = require("fs");
const path = require("path")
const http = require('http')
const encoder = new TextEncoder();
const decoder = new TextDecoder();
const jayson = require('jayson');

const client = jayson.client.http({
    port: 8545
  });


type BN = typeof BN;

// Global singleton object which remembers the ABI associated with each
// contract. This maps contract addresses => ABIs. Ethereum itself has a
// mechanism for this via swarm data etc., but for testing we want to emulate it
// here, particularly given pwasm support for such swarm may not be feature
// complete.
class ABICache {
    private map: Map<string, any>;
    constructor() {
        this.map = new Map();
    }

    public add(address: string, abi: any) {
        this.map.set(address, abi);
    }

    public get(address: string, abi: any): any {
        return this.map.get(address);
    }
}

const abi_cache = new ABICache();

// Get BuildPath
const TARGET_PATH = path.resolve(process.cwd(), './target');
// Get Dev Chain Config
export const CHAIN_CONFIG = require(path.resolve(process.cwd(), './wasm-dev-chain.json'));
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


// The default ABI of a cap9 kernel is the ABI of it's entry kernel. Often we
// will want to use a variety of other ABIs depending on which procedure we want
// to interact with. This API also uses direct storage reads to perform some
// tests. The necessarily reimplements some storage location logic accoding to
// the standard.
export class KernelInstance {
    private abi_cache: ABICache;
    constructor(public contract: Contract) {
        this.abi_cache = abi_cache;
    }

    public async getStorageAt(location: Uint8Array): Promise<Uint8Array> {
        const storageValue = await web3.eth.getStorageAt(this.contract.address, bufferToHex(location));
        return hexToBuffer(storageValue);
    }

    // Return the 24 bytes of the entry procedure key.
    public async getEntryProcedure(): Promise<Uint8Array> {
        const storageValue = await this.getStorageAt(KERNEL_ENTRY_PROC_PTR);
        return storageValue.slice(8,32);
    }

    // Return the 24 bytes of the current procedure key.
    public async getCurrentProcedure(): Promise<Uint8Array> {
        const storageValue = await this.getStorageAt(KERNEL_CURRENT_PROC_PTR);
        return storageValue.slice(8,32);
    }

    public async getNProcedures(): Promise<BN> {
        const storageValue = await this.getStorageAt(KERNEL_PROC_LIST_PTR);
        return web3.utils.toBN(bufferToHex(storageValue));
    }

    private getListPtr(index: number): Uint8Array {
        if (index > 255) {
            throw new Error("indices of greather than 255 not supported");
        }
        const ptr = KERNEL_PROC_LIST_PTR.slice();
        ptr[28] = index;
        return ptr;
    }

    public async getProcedureKey(index: number): Promise<Uint8Array> {
        const ptr = this.getListPtr(index);
        return this.getStorageAt(ptr).then(x=>x.slice(8,32));
    }

    public async getProcedures(): Promise<Array<Procedure>> {
        const procs: Array<Promise<Procedure>> = [];
        const nProcs = await this.getNProcedures().then(x=>x.toNumber());
        for (let i = 1; i <= nProcs; i++) {
            const ptr = this.getListPtr(i);
            const proc = this.getStorageAt(ptr).then(x=>x.slice(8,32)).then(async (key) => {
                // Get the pointer the the base of the Procedure Data in the
                // Procedure Heap.
                const proc_ptr = Uint8Array.from([0xff,0xff,0xff,0xff,0x00].concat(Array.from(key)).concat([0x00,0x00,0x00]))
                // Read the Ethereum address of the contract for this procedure.
                const address = await this.getStorageAt(proc_ptr).then(x=>x.slice(12,32));
                const index_ptr = Uint8Array.from([0xff,0xff,0xff,0xff,0x00].concat(Array.from(key)).concat([0x00,0x00,0x01]))
                const index = await this.getStorageAt(index_ptr)
                    .then(bufferToHex)
                    .then(web3.utils.hexToNumber);
                // Read the capabilities for this procedure.
                const caps = await Caps.from(this, key);
                // Return a new Procedure object.
                return new Procedure(key, address, index, caps);
            });
            procs.push(proc);
        }
        return Promise.all(procs);
    }

    public async getProcedureKeys(): Promise<Array<Uint8Array>> {
        const procs: Array<Promise<Uint8Array>> = [];
        const nProcs = await this.getNProcedures().then(x=>x.toNumber());
        for (let i = 1; i <= nProcs; i++) {
            procs.push(this.getProcedureKey(i));
        }
        return Promise.all(procs);
    }

    public async getProcedureKeysAscii(): Promise<Array<string>> {
        const procs = await this.getProcedureKeys();
        return procs.map(x=>decoder.decode(x));
    }

    public async listStorageKeys(n) {
        return new Promise((resolve,reject) => {
            client.request('parity_listStorageKeys', [this.contract.address, n], function(err, response) {
                if(err) reject(err);
                resolve(response.result);
            });
        });
    }

    // TODO: deprecate
    async getProcCapTypeLen(proc_key: string, cap_type: CAP_TYPE): Promise<number> {
        return utils.toDecimal(await this.contract.methods.get_cap_type_len(proc_key, cap_type).call());
    }

    // TODO: every time we add a contract to the kernel, that contract address
    // is immutably associated with an ABI, so we can cache those.

}

// WARNING: this is a lossy conversion, and should be used for
// display/convinience only.
export function bufferToString(buffer: Uint8Array) {
    const nullIndex = buffer.indexOf(0x00);
    const sliceIndex = nullIndex ? nullIndex : undefined;
    return decoder.decode(buffer.slice(0,sliceIndex));
}

export function bufferToHex(buffer: Uint8Array) {
    return web3.utils.bytesToHex(Array.from<number>(buffer));
}

export function hexToBuffer(str: string): Uint8Array {
    return Uint8Array.from(web3.utils.hexToBytes(str));
}

const KERNEL_PROC_HEAP_PTR: Uint8Array = new Uint8Array([
    0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
]);

const KERNEL_PROC_LIST_PTR: Uint8Array = new Uint8Array([
    0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
]);

const KERNEL_ADDRESS_PTR: Uint8Array = new Uint8Array([
    0xff, 0xff, 0xff, 0xff, 0x02, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
]);

const KERNEL_CURRENT_PROC_PTR: Uint8Array = new Uint8Array([
    0xff, 0xff, 0xff, 0xff, 0x03, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
]);

const KERNEL_ENTRY_PROC_PTR: Uint8Array = new Uint8Array([
    0xff, 0xff, 0xff, 0xff, 0x04, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
]);

// A representation of a procedure in the kernel.
export class Procedure {
    constructor(public key: Uint8Array, public address: Uint8Array, public index: number, public caps: Caps) {

    }

    public toString() {
        let str = "";
        str += `Procedure[${this.index}]:\n`;
        str += `  Key: ("${bufferToString(this.key)}") ${bufferToHex(this.key)}\n`;
        str += `  Address: ${bufferToHex(this.address)}\n`;
        str += `  ${this.caps.toString()}\n`;
        return str;
    }
}

export class Caps {
    constructor (public writeCaps: WriteCap[], public regCaps: RegisterCap[]) {

    }

    static async from(kernel: KernelInstance, key: Uint8Array) {
        const write_ptr = Uint8Array.from([0xff,0xff,0xff,0xff,0x00].concat(Array.from(key)).concat([CAP_TYPE.STORE_WRITE,0x00,0x00]))
        const n_write_caps = await kernel.getStorageAt(write_ptr)
            .then(bufferToHex)
            .then(web3.utils.hexToNumber);
        const writeCaps = [];
        for (let index = 0; index < n_write_caps; index++) {
            const writeCap = await WriteCap.from(kernel, key, index);
            writeCaps.push(writeCap);
        }

        const reg_ptr = Uint8Array.from([0xff,0xff,0xff,0xff,0x00].concat(Array.from(key)).concat([CAP_TYPE.PROC_REGISTER,0x00,0x00]))
        const n_reg_caps = await kernel.getStorageAt(reg_ptr)
            .then(bufferToHex)
            .then(web3.utils.hexToNumber);
        const regCaps = [];
        for (let index = 0; index < n_reg_caps; index++) {
            const regCap = await RegisterCap.from(kernel, key, index);
            regCaps.push(regCap);
        }
        return new Caps(writeCaps, regCaps);
    }

    toString() {
        return `Caps:
    WRITE(${this.writeCaps.length}):\n${this.writeCaps.map(x=> `      ${x.toString()}`).join("\n")}
    REGISTER(${this.writeCaps.length}):\n${this.regCaps.map(x=>`      ${x.toString()}`).join("\n")}`
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
    to_input(): (string | number)[] {
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
        const baseKeyHex = utils.toHex(baseKey24).slice(2,50);
        // prefixHex is the prefix length hex-encoded and padded to two chars (a
        // single byte). The "0x" is removed here also.
        const prefixHex = utils.toHex(this.prefixLength).slice(2).padStart(2,'0');
        // There are 7 bytes between the prefix length and the start of the base
        // key.
        const undefinedFill = "".padEnd(14,'0');
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
        const baseKeyHex = utils.toHex(baseKey24).slice(2,50);
        // prefixHex is the prefix length hex-encoded and padded to two chars (a
        // single byte). The "0x" is removed here also.
        const prefixHex = utils.toHex(this.prefixLength).slice(2).padStart(2,'0');
        // There are 7 bytes between the prefix length and the start of the base
        // key.
        const undefinedFill = "".padEnd(14,'0');
        // We string these together in the correct order.
        const key = "0x" + prefixHex + undefinedFill + baseKeyHex;

        return [key as any]
    }
    toString(): string {
        const baseKey = hexToBuffer(this.baseKey);
        return `RegProc
        prefix: ${this.prefixLength}
        key: ("${bufferToString(baseKey)}") ${bufferToHex(baseKey)}`;
    }

    static async from(kernel: KernelInstance, key: Uint8Array, index: number): Promise<RegisterCap> {
        const value_ptr = Uint8Array.from([0xff,0xff,0xff,0xff,0x00]
            .concat(Array.from(key))
            .concat([CAP_TYPE.PROC_REGISTER,index+1,0x00]));
        const value = await kernel.getStorageAt(value_ptr);
        const prefix = value[0];
        const cap_key = value.slice(8,32);
        return new RegisterCap(prefix, bufferToHex(cap_key));
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
        const baseKeyHex = utils.toHex(baseKey24).slice(2,50);
        // prefixHex is the prefix length hex-encoded and padded to two chars (a
        // single byte). The "0x" is removed here also.
        const prefixHex = utils.toHex(this.prefixLength).slice(2).padStart(2,'0');
        // There are 7 bytes between the prefix length and the start of the base
        // key.
        const undefinedFill = "".padEnd(14,'0');
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
    // TODO: these types are not quite correct
    constructor(public location: number, public size: number) { }
    to_input(): number[] {
        return [this.location, this.size]
    }
    toString(): string {
        return `WriteCap
        location: ${this.location}
        size: ${this.size}`;
    }

    static async from(kernel: KernelInstance, key: Uint8Array, index: number): Promise<WriteCap> {
        const location_ptr = Uint8Array.from([0xff,0xff,0xff,0xff,0x00].concat(Array.from(key)).concat([CAP_TYPE.STORE_WRITE,index+1,0x00]))
        const size_ptr = Uint8Array.from([0xff,0xff,0xff,0xff,0x00].concat(Array.from(key)).concat([CAP_TYPE.STORE_WRITE,index+1,0x01]))
        const location_p = kernel.getStorageAt(location_ptr)
            .then(bufferToHex)
            ;
        const size_p = kernel.getStorageAt(size_ptr)
            .then(bufferToHex)
            ;
        const [location, size] = await Promise.all([location_p, size_p]);
        return new WriteCap(location, size);
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
        return [this.topics.length].concat([topic1, topic2, topic3, topic4].map(x=>web3.utils.fromAscii(x,32)) as any)
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
                resolve(JSON.parse(chunk).result);
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

export async function deployContract(file_name: string, abi_name: string, args?: []): Promise<Contract> {
    // Create Account
    const newAccount = await createAccount(DEFAULT_ACCOUNT.NAME, DEFAULT_ACCOUNT.PASSWORD);
    const accounts = await web3.eth.personal.getAccounts();
    if (accounts.length == 0)
        throw `Got zero accounts`;

    const account = web3.utils.toChecksumAddress(newAccount, web3.utils.hexToNumber(CHAIN_CONFIG.params.networkId));
    web3.eth.defaultAccount = account;

    // read JSON ABI
    const abi = JSON.parse(fs.readFileSync(path.resolve(TARGET_PATH, `./json/${abi_name}.json`)));

    // convert Wasm binary to hex format
    const codeHex = '0x' + fs.readFileSync(path.resolve(TARGET_PATH, `./${file_name}.wasm`)).toString('hex');
    const Contract = new web3.eth.Contract(abi, null, { data: codeHex, from: account, transactionConfirmationBlocks: 1 } as any);
    const DeploymentTx = Contract.deploy({ data: codeHex, arguments: args });

    await web3.eth.personal.unlockAccount(account, DEFAULT_ACCOUNT.PASSWORD, null);
    let gas = await DeploymentTx.estimateGas();
    let contract_tx = DeploymentTx.send({ gasLimit: gas, from: account } as any);
    let tx_hash: string = await new Promise((res, rej) => contract_tx.on('transactionHash', res).on('error', rej));
    let tx_receipt = await web3.eth.getTransactionReceipt(tx_hash);
    let contract_addr = tx_receipt.contractAddress;
    let contract = Contract.clone();
    contract.address = contract_addr;
    abi_cache.add(contract.address, contract);
    return contract;
}

export async function newKernelInstance(proc_key: string, proc_address: string, cap_list: NewCap[] = [], initial_balance: number = 0): Promise<KernelInstance> {
    // Create Account
    const newAccount = await createAccount(DEFAULT_ACCOUNT.NAME, DEFAULT_ACCOUNT.PASSWORD);
    const accounts = await web3.eth.personal.getAccounts();
    if (accounts.length == 0)
        throw `Got zero accounts`;
    const account = web3.utils.toChecksumAddress(newAccount, web3.utils.hexToNumber(CHAIN_CONFIG.params.networkId));
    web3.eth.defaultAccount = account;
    const abi = KERNEL_WASM_ABI
    const codeHex = KERNEL_WASM

    // Encode CapList
    let encoded_cap_list: string[] = cap_list.reduce((payload, cap) => payload.concat(cap.to_input()), []);

    const KernelContract = new web3.eth.Contract(abi, null, { data: codeHex, from: account, transactionConfirmationBlocks: 1 } as any);
    const TokenDeployTransaction = KernelContract.deploy({ data: codeHex, arguments: [proc_key, proc_address, encoded_cap_list] });
    await web3.eth.personal.unlockAccount(account, DEFAULT_ACCOUNT.PASSWORD, null);
    let gas = await TokenDeployTransaction.estimateGas();
    let contract_tx = TokenDeployTransaction.send({ gasLimit: gas, from: account, value: initial_balance } as any);
    let tx_hash: string = await new Promise((res, rej) => contract_tx.on('transactionHash', res).on('error', rej));
    let tx_receipt = await web3.eth.getTransactionReceipt(tx_hash);
    let contract_addr = tx_receipt.contractAddress;
    let contract = KernelContract.clone();
    contract.address = contract_addr;
    return new KernelInstance(contract);
}


// Given a web3 value, normalize it so we can compare easily.
export function normalize(value) {
    return web3.utils.toHex(web3.utils.toBN(value))
}
