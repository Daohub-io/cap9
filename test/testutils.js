const beakerlib = require("../beakerlib");
const BasicEntryProcedure = artifacts.require('BasicEntryProcedure.sol');

async function installEntryProc(kernel) {
    const entryProcName = "EntryProcedure";
    kernel.setEntryProcedure(entryProcName);
    const capArrayEntryProc = beakerlib.Cap.toInput([
        new beakerlib.WriteCap(0x8001,2),
        new beakerlib.LogCap([]),
        new beakerlib.CallCap()
    ]);
    const deployedEntryProc = await deployedTrimmed(BasicEntryProcedure);
    // Install the entry procedure
    await kernel.registerAnyProcedure(entryProcName, deployedEntryProc.address, capArrayEntryProc);
}
exports.installEntryProc = installEntryProc;

function trimSwarm(bytecode) {
    const size = bytecode.length;
    const swarmSize = 43; // bytes
    // overwrite the swarm data with '0'
    return bytecode.slice(0, size - (swarmSize*2)).padEnd(size,'0');
}
exports.trimSwarm = trimSwarm;

async function deployedTrimmed(contract) {
    const bytecode = trimSwarm(contract.bytecode);
    return new contract.web3.eth.Contract(contract.abi).deploy({data: bytecode}).send(contract.class_defaults)
}
exports.deployedTrimmed = deployedTrimmed;
