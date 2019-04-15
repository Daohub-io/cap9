const beakerlib = require("../beakerlib");
const BasicEntryProcedure = artifacts.require('BasicEntryProcedure.sol');
const Kernel = artifacts.require('./TestKernel.sol')

// Deploy a kernel and install the example entry procedure
async function deployTestKernel(entryProc = BasicEntryProcedure) {
    // First deploy the entry procedure that will be used to bootstrap the
    // system.
    const deployedEntryProc = await deployedTrimmed(entryProc);

    // Deploy the kernel, specifying the previsouly deployed procedure as the
    // first entry procedure. This will be named "init                    ".
    let kernel;
    try {
        kernel = await Kernel.new(deployedEntryProc.address);
    } catch (e) {
        throw new Error(e);
    }

    const procedures1Raw = await kernel.listProcedures.call();
    const procedures1 = procedures1Raw.map(web3.toAscii)
        .map(s => s.replace(/\0.*$/, ''));
    assert(procedures1.length == 1,
        "The kernel should initially have a single procedure procedures");
    const initEntryProc = await kernel.getEntryProcedure.call();
    // Check that the entry procedure was correctly installed.
    assert.strictEqual(web3.toAscii(initEntryProc).replace(/\0.*$/, ''), "init",
        "The kernel should have an entry procedure registered as \"init\"");
    // Check that the capabilities are correct
    // TODO: we want to ensure that the maximal caps are provided
    // await checkCaps(kernel, "init", []);
    return kernel;
}
exports.deployTestKernel = deployTestKernel;

function trimSwarm(bytecode) {
    const size = bytecode.length;
    const swarmSize = 43; // bytes
    // overwrite the swarm data with '0'
    return bytecode.slice(0, size - (swarmSize*2)).padEnd(size,'0');
}
exports.trimSwarm = trimSwarm;

async function deployedTrimmed(contract) {
    const bytecode = trimSwarm(contract.bytecode);
    return await contract.new({data:bytecode});
}
exports.deployedTrimmed = deployedTrimmed;

function validateBytecode(bytecode) {
    const code = [];
    let n = 0;
    // Parse the hex into integers. By keeping this in two stages we ensure that
    // the validation code mirrors that in the kernel code as much as possible.
    for (let i = 2; i < bytecode.length; i += 2) {
        const s = "0x"+bytecode[i]+bytecode[i+1];
        const ins = parseInt(s);
        code[n] = ins;
        n++
    }
    for (let i = 0; i < code.length; i++) {
        const ins = code[i];
        if((ins >= 0x00 && ins <= 0x0b)){continue;}  // Stop and Arithmetic
        if((ins >= 0x10 && ins <= 0x1a)){continue;} // Comparison & Bitwise Logic Operations
        if((ins == 0x20)){continue;} // SHA3
        if((ins >= 0x30 && ins <= 0x3e)){continue;} // Environmental Informatio
        if((ins >= 0x40 && ins <= 0x45)){continue;} // Block Information
        if((ins >= 0x50 && ins <= 0x53)){continue;} // Stack, Memory, Storage and Flow Operation
        if((ins >= 0x56 && ins <= 0x5b)){continue;} // Stack, Memory, Storage and Flow Operation
        if((ins >= 0x80 && ins <= 0x8f)){continue;} // Duplication Operations
        if((ins >= 0x90 && ins <= 0x9f)){continue;} // Exchange Operations
        if((ins == 0xf3)){continue;} // RETURN
        if((ins >= 0xfa && ins <= 0xfe)){continue;}

        if (ins >= 0x60 && ins <= 0x7f) {
            i += ins - 95;
            continue;
        } // PUSH
        // if (ins == 0x54) {return 1;} // SLOAD
        // TODO: we temporarily allow SLOAD for testing purposes
        if (ins == 0x54) {continue;} // SLOAD
        if (ins == 0x55) {return 2;} // SSTORE

        if (ins == 0xa0) {return 3;} // LOG0
        if (ins == 0xa1) {return 4;} // LOG1
        if (ins == 0xa2) {return 5;} // LOG2
        if (ins == 0xa3) {return 6;} // LOG3
        if (ins == 0xa4) {return 7;} // LOG4

        if (ins == 0xf0) {return 8;} // CREATE
        if (ins == 0xf1) {return 9;} // CALL
        if (ins == 0xf2) {return 10;} // CALLCODE
        if (ins == 0xf4) {
            // continue if it is a compliant syscall
            let isSysCall = false;
            // check there are enough bytes
            if (i < 2) {
                isSysCall = false;
            } else {
                isSysCall = (code[i-1] == 0x5a /* GAS */) && (code[i-2] == 0x33 /* CALLER */);
            }
            if (isSysCall) {
                continue;
            } else {
                return 11;
            }
        } // DELEGATECALL
        if (ins == 0xf5) {return 12;} // CREATE2
        if (ins == 0xff) {return 13;} // SELFDESTRUCT

        return 100; // UNKNOWN OPCODE

    }
    return 0;
}
