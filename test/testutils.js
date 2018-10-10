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
    const deployedEntryProc = await BasicEntryProcedure.new();
    // Install the entry procedure
    await kernel.registerAnyProcedure(entryProcName, entryProcBytecode, capArrayEntryProc);
}
exports.installEntryProc = installEntryProc;
