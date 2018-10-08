const beakerlib = require("../beakerlib");
const BasicEntryProcedure = artifacts.require('BasicEntryProcedure.sol');

async function installEntryProc(kernel) {
    const entryProcName = "EntryProcedure";
    kernel.setEntryProcedure(entryProcName);
    const entryProcBytecode = BasicEntryProcedure.bytecode;
    const capArrayEntryProc = beakerlib.Cap.toInput([
        new beakerlib.WriteCap(0x8001,2),
        new beakerlib.LogCap([]),
        new beakerlib.CallCap()
    ]);

    // Install the entry procedure
    await kernel.createAnyProcedure(entryProcName, entryProcBytecode, capArrayEntryProc);
}
exports.installEntryProc = installEntryProc;
