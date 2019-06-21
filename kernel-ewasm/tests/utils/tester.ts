// This module holds the Tester class, which contains most of the boilerplate
// required in tests.
import { Contract } from "web3-eth-contract";
import * as utils from 'web3-utils';
import { newKernelInstance, web3, createAccount, KernelInstance, deployContract, normalize, EntryCap, WriteCap, RegisterCap, NewCap} from '../utils'

const assert = require('assert')

const BN = require('bn.js')
const Web3 = require('web3');
const fs = require("fs");
const path = require("path")
const http = require('http')

// The tester class is an object for executing tests on the kernel.
export class Tester {
    entry_proc: TestContract = null;
    entry_proc_name: string;
    kernel: any;
    interface: any;
    constructor() {
    }

    // Configure the entry procedure that will be used on first deployment of
    // the kernel. This procedure will be deployed during init(). This should be
    // used before init().
    setFirstEntry(name: string, contract: TestContract) {
        this.entry_proc_name = name;
        this.entry_proc = contract;
    }

    // Deploy the kernel and any initial entry procedure. This also creates the
    // interface for the kernel using the ABI of the entry procedure.
    async init() {
        if (this.entry_proc === null) {
            throw new Error("no entry proc has been set")
        }
        // Deploy the first entry procedure
        const firstEntry = await deployContract(this.entry_proc.name, this.entry_proc.abiName);
        this.kernel = await newKernelInstance(this.entry_proc_name, firstEntry.address, this.entry_proc.caps);
        // Here we make a copy of the entry procedure contract interface, but
        // change the address so that it's pointing at the kernel. This
        // means the web3 library will send a message crafted to be read by
        // the writer contract directly to the kernel.
        this.interface = firstEntry.clone();
        this.interface.address = this.kernel.contract.address;
        // In order to execute procedures, we first have to put the kernel into
        // "entry procedure mode".
        const toggle1 = await this.kernel.contract.methods.get_mode().call();
        assert.strictEqual(toggle1, 0, "The kernel should be in test mode (0)");
        await this.kernel.contract.methods.toggle_syscall().send();
    }

    // Register a procedure. This assumes the current entry procedure for the
    // kernel provides the following interface:
    //
    //    * fn regProc(&mut self, cap_idx: U256, key: H256, address: Address,
    //      cap_list: Vec<H256>);
    //    * fn listProcs(&mut self) -> Vec<H256>;
    //    * fn getCap(&mut self, cap_type: U256, cap_index: U256) -> (U256,
    //      U256);
    //    * fn getNCaps(&mut self, key: H256) -> u64;
    //
    // This method will also execute tests to ensure that the registration
    // occurs successfully.
    async registerTest(requestedCaps, procName, contractName, contractABIName, result) {
        // This is the key of the procedure that we will be registering.
        const key = "0x" + web3.utils.fromAscii(procName, 24).slice(2).padStart(64, "0");
        // This is the index of the capability (in the procedures capability
        // list) that we will be using to perform the writes.
        const cap_index = 0;

        // Deploy the contract for the procedure that we will register.
        let writeProc = await deployContract(contractName, contractABIName);
        const writer_caps = requestedCaps;
        const encoded_writer_caps = writer_caps.reduce((payload, cap) => payload.concat(cap.to_input()), []);
        // This is the address of the new procedure that we wish to register.

        const procList1 = await this.interface.methods.listProcs().call().then(x=>x.map(normalize));
        // We then send that message via a call procedure syscall.
        const message = this.interface.methods.regProc(cap_index, key, writeProc.address, encoded_writer_caps).encodeABI();
        if (result) {
            // The transaction should succeed
            const return_value = await web3.eth.sendTransaction({ to: this.kernel.contract.address, data: message });
            const procList2 = await this.interface.methods.listProcs().call().then(x=>x.map(normalize));
            assert.strictEqual(procList2.length, procList1.length + 1, "The number of procedures should have increased by 1");
            assert(procList2.includes(normalize(web3.utils.fromAscii(procName,24))), "The new procedure key should be included in the table");

            // Check that the new procedure has the correct caps.
            // TODO: update for other cap types.
            const resulting_caps = await this.interface.methods.getNCaps(web3.utils.fromAscii("write",24)).call();
            assert.strictEqual(normalize(resulting_caps), normalize(requestedCaps.length), "The requested number of write caps should be written");
        } else {
            // The transaction should not succeed
            let success;
            try {
                const return_value = await web3.eth.sendTransaction({ to: this.kernel.contract.address, data: message });
                success = true;
            } catch (e) {
                success = false;
            }
            assert(!success, "Call should not succeed");
            const procList2 = await this.interface.methods.listProcs().call().then(x=>x.map(normalize));
            assert.strictEqual(procList2.length, procList1.length, "The number of procedures should not have increased");
            assert(!procList2.includes(normalize(web3.utils.fromAscii(procName,24))), "The new procedure key should not be included in the table");
        }
    }

    // Set an entry procedure. This assumes the current entry procedure for the
    // kernel provides the following interface:
    //
    //    * fn setEntry(&mut self, cap_idx: U256, key: H256);
    //    * fn getEntry(&mut self) -> H256;
    //
    // This method will also execute tests to ensure that the registration
    // occurs successfully.
    async setEntryTest(procName, result) {
        const old_entry_proc = await this.interface.methods.getEntry().call();
        // This is the key of the procedure that we will be setting to the entry
        // procedure.
        const key = "0x" + web3.utils.fromAscii(procName, 24).slice(2).padStart(64, "0");
        // This is the index of the capability (in the procedures capability
        // list) that we will be using to perform the writes.
        const cap_index = 0;
        // We then send that message via a call procedure syscall.
        const message = this.interface.methods.setEntry(cap_index, key).encodeABI();

        if (result) {
            // The transaction should succeed
            const return_value = await web3.eth.sendTransaction({ to: this.kernel.contract.address, data: message });
            const new_entry_proc = await this.interface.methods.getEntry().call();
            assert.strictEqual(normalize(new_entry_proc), normalize(key), "The entry proc should be set to the requested key");
        } else {
            // The transaction should not succeed
            let success;
            try {
                const return_value = await web3.eth.sendTransaction({ to: this.kernel.contract.address, data: message });
                success = true;
            } catch (e) {
                success = false;
            }
            assert(!success, "Call should not succeed");
            const new_entry_proc = await this.interface.methods.getEntry().call();
            assert.strictEqual(normalize(new_entry_proc), normalize(old_entry_proc), "The entry proc should remain the same");
        }
    }

}

export class TestContract {
    public name: string;
    public abiName: string;
    public caps: NewCap[];
    constructor (name, abiName, caps) {
        this.name = name;
        this.abiName = abiName;
        this.caps = caps;
    }
}
