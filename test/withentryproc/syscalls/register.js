const debug = require('debug')
const assert = require('assert')

const Kernel = artifacts.require('./TestKernel.sol')
const abi = require('ethereumjs-abi')

const beakerlib = require("../../../beakerlib");
const testutils = require("../../testutils.js");

// Valid Contracts
const Valid = {
    Adder: artifacts.require('test/valid/Adder.sol'),
    Multiply: artifacts.require('test/valid/Multiply.sol'),
    Divide: artifacts.require('test/valid/Divide.sol'),
    SysCallTestWrite: artifacts.require('test/valid/SysCallTestWrite.sol'),
    SysCallTestCall: artifacts.require('test/valid/SysCallTestCall.sol'),
    SysCallTestProcRegister: artifacts.require('test/valid/SysCallTestProcRegister.sol'),
    BasicEntryProcedure: artifacts.require('BasicEntryProcedure.sol'),
}

const TestWrite = artifacts.require('test/TestWrite.sol');

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

contract('Kernel with entry procedure', function () {
    // These tests have a general form in which there are 2 procedures,
    // Procedure A and Procedure B. The following will be performed for each
    // test, although it isn't part of the properties being tested:
    //    1. A new kernel will be deployed.
    //    2. A basic entry procedure will be installed.
    //    3. The contract code for Procedure A and Procedure B will be deployed
    //       to the chain.
    //    4. Procedure A will be registered with the kernel.
    //
    // We will then test procedure registration by getting Procedure A to
    // register Procedure B with a set of capabilities.
    describe('Register without capabilities', function () {
        it('Should succeed when Procedure A is given a general Register Capability', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.WriteCap(0x8000,2),
                new beakerlib.RegisterCap(0, ""),
                new beakerlib.CallCap(0,""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [];

            const shouldSucceed = true;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should succeed when Procedure A is given a Register Capability with the same name as ProcB and 192-bit prefix', async function () {
            const procAName = "SysCallTestProcRegister";
            const procBName = "Adder";

            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.WriteCap(0x8000,2),
                new beakerlib.RegisterCap(192, procBName),
                new beakerlib.CallCap(0,""),
            ];

            const procBContract = Valid.Adder;
            const procBCaps = [];

            const shouldSucceed = true;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should succeed when Procedure A is given a Register Capability with the same name as ProcB and 30-bit prefix', async function () {
            const procAName = "SysCallTestProcRegister";
            const procBName = "Adder";

            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.WriteCap(0x8000,2),
                new beakerlib.RegisterCap(30, procBName),
                new beakerlib.CallCap(0,""),
            ];

            const procBContract = Valid.Adder;
            const procBCaps = [];

            const shouldSucceed = true;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should succeed when Procedure A is given a Register Capability with the first 2 bytes of as ProcBName and 16-bit prefix', async function () {
            const procAName = "SysCallTestProcRegister";
            const procBName = "Adder";

            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.WriteCap(0x8000,2),
                new beakerlib.RegisterCap(16, "Ad"),
                new beakerlib.CallCap(0,""),
            ];

            const procBContract = Valid.Adder;
            const procBCaps = [];

            const shouldSucceed = true;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should fail when Procedure A is given a Register Capability with the first 2 bytes of as ProcBName and 24-bit prefix', async function () {
            const procAName = "SysCallTestProcRegister";
            const procBName = "Adder";

            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.WriteCap(0x8000,2),
                new beakerlib.RegisterCap(24, "Ad"),
                new beakerlib.CallCap(0,""),
            ];

            const procBContract = Valid.Adder;
            const procBCaps = [];

            const shouldSucceed = false;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should fail when Procedure A is not given any Register Capability', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.WriteCap(0x8000,2),
                new beakerlib.CallCap(0,""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [];

            const shouldSucceed = false;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
    });
    describe('Register Call capability', async function () {
        await testCallType(beakerlib.CallCap);
    });
    describe('Register Delete capability', async function () {
        await testCallType(beakerlib.DeleteCap);
    });
    describe('Register Register capability', async function () {
        await testCallType(beakerlib.RegisterCapWithIndexOne);
    });
    describe('Register Write capability', function () {
        it('Should succeed when deriving maximal cap from maximal cap', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.WriteCap(0x00,"0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe"),
                new beakerlib.RegisterCap(0, ""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [
                new beakerlib.WriteCap(0x00,"0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe"),
            ];

            const shouldSucceed = true;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should succeed when deriving invalid cap from maximal cap', async function () {
            // This looks at when size extends far beyond available storage.
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.WriteCap(0x00,"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
                new beakerlib.RegisterCap(0, ""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [
                new beakerlib.WriteCap(0xffffffff,"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            ];

            const shouldSucceed = true;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should succeed when reduced cap from maximal cap', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.WriteCap(0x00,"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
                new beakerlib.RegisterCap(0, ""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [
                new beakerlib.WriteCap(0x80,100),
            ];

            const shouldSucceed = true;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should fail when reduced cap from cap, base address ok, size too big', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.WriteCap(0x8000,100),
                new beakerlib.RegisterCap(0, ""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [
                new beakerlib.WriteCap(0x8000,101),
            ];

            const shouldSucceed = false;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should fail when reduced cap from cap, base address too low, size ok', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.WriteCap(0x8000, 100),
                new beakerlib.RegisterCap(0, ""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [
                new beakerlib.WriteCap(0x7000,30),
            ];

            const shouldSucceed = false;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should fail when reduced cap from cap, base address too high, size ok', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.WriteCap(0x8000, 100),
                new beakerlib.RegisterCap(0, ""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [
                new beakerlib.WriteCap(0x9000,30),
            ];

            const shouldSucceed = false;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
    });
    describe('Register Log capability', function () {
        it('Should succeed when deriving maximal cap from maximal cap', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.LogCap([]),
                new beakerlib.RegisterCap(0, ""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [
                new beakerlib.LogCap([]),
            ];

            const shouldSucceed = true;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should succeed when adding one topic to cap from maximal cap', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.LogCap([]),
                new beakerlib.RegisterCap(0, ""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [
                new beakerlib.LogCap([web3.fromAscii("abcd")]),
            ];

            const shouldSucceed = true;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should succeed when adding two topics to cap from maximal cap', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.LogCap([]),
                new beakerlib.RegisterCap(0, ""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [
                new beakerlib.LogCap([web3.fromAscii("abcd"), web3.fromAscii("efgh")]),
            ];

            const shouldSucceed = true;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should succeed when adding three topics to cap from maximal cap', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.LogCap([]),
                new beakerlib.RegisterCap(0, ""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [
                new beakerlib.LogCap([web3.fromAscii("abcd"), web3.fromAscii("efgh"), web3.fromAscii("ijkl")]),
            ];

            const shouldSucceed = true;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should succeed when adding four topics to cap from maximal cap', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.LogCap([]),
                new beakerlib.RegisterCap(0, ""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [
                new beakerlib.LogCap([web3.fromAscii("abcd"), web3.fromAscii("efgh"), web3.fromAscii("ijkl"), web3.fromAscii("mnop")]),
            ];

            const shouldSucceed = true;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should succeed when deriving one topic from same one topic cap', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.LogCap([web3.fromAscii("abcd")]),
                new beakerlib.RegisterCap(0, ""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [
                new beakerlib.LogCap([web3.fromAscii("abcd")]),
            ];

            const shouldSucceed = true;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should succeed when deriving one topic from two topic cap, even though that top is the same', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.LogCap([web3.fromAscii("abcd"), web3.fromAscii("efgh")]),
                new beakerlib.RegisterCap(0, ""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [
                new beakerlib.LogCap([web3.fromAscii("abcd")]),
            ];

            const shouldSucceed = false;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
        it('Should fail when deriving one topic from differnt one topic cap', async function () {
            const procAName = "SysCallTestProcRegister";
            const procAContract = Valid.SysCallTestProcRegister;
            const procACaps = [
                new beakerlib.LogCap([web3.fromAscii("abcd")]),
                new beakerlib.RegisterCap(0, ""),
            ];

            const procBName = "Adder";
            const procBContract = Valid.Adder;
            const procBCaps = [
                new beakerlib.LogCap([web3.fromAscii("efgh")]),
            ];

            const shouldSucceed = false;
            await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
        });
    });
})

async function testCallType(ThisCap) {
    it('Should succeed when deriving general cap from general cap', async function () {
        const procAName = "SysCallTestProcRegister";
        const procAContract = Valid.SysCallTestProcRegister;
        const procACaps = [
            new beakerlib.WriteCap(0x8000,2),
            new beakerlib.RegisterCap(0, ""),
            new ThisCap(0,""),
        ];

        const procBName = "Adder";
        const procBContract = Valid.Adder;
        const procBCaps = [
            new ThisCap(0,""),
        ];

        const shouldSucceed = true;
        await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
    });
    it('Should fail when deriving general cap from no cap', async function () {
        const procAName = "SysCallTestProcRegister";
        const procAContract = Valid.SysCallTestProcRegister;
        const procACaps = [
            new beakerlib.WriteCap(0x8000,2),
            new beakerlib.RegisterCap(0, ""),
        ];

        const procBName = "Adder";
        const procBContract = Valid.Adder;
        const procBCaps = [
            new ThisCap(0,""),
        ];

        const shouldSucceed = false;
        await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
    });
    it('Should succeed when deriving larger prefix cap (30) from general cap', async function () {
        const procAName = "SysCallTestProcRegister";
        const procAContract = Valid.SysCallTestProcRegister;
        const procACaps = [
            new beakerlib.WriteCap(0x8000,2),
            new beakerlib.RegisterCap(0, ""),
            new ThisCap(0,""),
        ];

        const procBName = "Adder";
        const procBContract = Valid.Adder;
        const procBCaps = [
            new ThisCap(30,""),
        ];

        const shouldSucceed = true;
        await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
    });
    it('Should succeed when deriving larger prefix cap (192) from general cap', async function () {
        const procAName = "SysCallTestProcRegister";
        const procAContract = Valid.SysCallTestProcRegister;
        const procACaps = [
            new beakerlib.WriteCap(0x8000,2),
            new beakerlib.RegisterCap(0, ""),
            new ThisCap(0,""),
        ];

        const procBName = "Adder";
        const procBContract = Valid.Adder;
        const procBCaps = [
            new ThisCap(192,""),
        ];

        const shouldSucceed = true;
        await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
    });
    it('Should fail when deriving smaller prefix cap (30) from larger prefix cap (48)', async function () {
        const procAName = "SysCallTestProcRegister";
        const procAContract = Valid.SysCallTestProcRegister;
        const procACaps = [
            new beakerlib.WriteCap(0x8000,2),
            new beakerlib.RegisterCap(0, ""),
            new ThisCap(48,""),
        ];

        const procBName = "Adder";
        const procBContract = Valid.Adder;
        const procBCaps = [
            new ThisCap(30,""),
        ];

        const shouldSucceed = false;
        await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
    });
    it('Should succeed when deriving 2 larger prefix caps (40, 192) from general cap', async function () {
        const procAName = "SysCallTestProcRegister";
        const procAContract = Valid.SysCallTestProcRegister;
        const procACaps = [
            new beakerlib.WriteCap(0x8000,2),
            new beakerlib.RegisterCap(0, ""),
            new ThisCap(0,""),
        ];

        const procBName = "Adder";
        const procBContract = Valid.Adder;
        const procBCaps = [
            new ThisCap(40,""),
            new ThisCap(192,""),
        ];

        const shouldSucceed = true;
        await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
    });
    it('Should succeed when deriving general cap with address from general cap', async function () {
        const procAName = "SysCallTestProcRegister";
        const procAContract = Valid.SysCallTestProcRegister;
        const procACaps = [
            new beakerlib.WriteCap(0x8000,2),
            new beakerlib.RegisterCap(0, ""),
            new ThisCap(0,""),
        ];

        const procBName = "Adder";
        const procBContract = Valid.Adder;
        const procBCaps = [
            new ThisCap(0,"abcd"),
        ];

        const shouldSucceed = true;
        await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
    });
    it('Should succeed when deriving prefix(30) with address from general cap', async function () {
        const procAName = "SysCallTestProcRegister";
        const procAContract = Valid.SysCallTestProcRegister;
        const procACaps = [
            new beakerlib.WriteCap(0x8000,2),
            new beakerlib.RegisterCap(0, ""),
            new ThisCap(0,""),
        ];

        const procBName = "Adder";
        const procBContract = Valid.Adder;
        const procBCaps = [
            new ThisCap(30,"abcd"),
        ];

        const shouldSucceed = true;
        await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
    });
    it('Should succeed when deriving 2 general caps with different addresses from general cap', async function () {
        const procAName = "SysCallTestProcRegister";
        const procAContract = Valid.SysCallTestProcRegister;
        const procACaps = [
            new beakerlib.WriteCap(0x8000,2),
            new beakerlib.RegisterCap(0, ""),
            new ThisCap(0,""),
        ];

        const procBName = "Adder";
        const procBContract = Valid.Adder;
        const procBCaps = [
            new ThisCap(0,"abcd"),
            new ThisCap(0,"wxyz"),
        ];

        const shouldSucceed = true;
        await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
    });
    it('Should succeed when deriving 2 prefix(30) caps with different addresses from general cap', async function () {
        const procAName = "SysCallTestProcRegister";
        const procAContract = Valid.SysCallTestProcRegister;
        const procACaps = [
            new beakerlib.WriteCap(0x8000,2),
            new beakerlib.RegisterCap(0, ""),
            new ThisCap(0,""),
        ];

        const procBName = "Adder";
        const procBContract = Valid.Adder;
        const procBCaps = [
            new ThisCap(30,"abcd"),
            new ThisCap(30,"wxyz"),
        ];

        const shouldSucceed = true;
        await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
    });
    it('Should succeed when deriving a prefix(16) cap and a prefix(32) cap with different addresses from general cap', async function () {
        const procAName = "SysCallTestProcRegister";
        const procAContract = Valid.SysCallTestProcRegister;
        const procACaps = [
            new beakerlib.WriteCap(0x8000,2),
            new beakerlib.RegisterCap(0, ""),
            new ThisCap(0,""),
        ];

        const procBName = "Adder";
        const procBContract = Valid.Adder;
        const procBCaps = [
            new ThisCap(16,"ab"),
            new ThisCap(32,"wxyz"),
        ];

        const shouldSucceed = true;
        await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
    });
    it('Should succeed when deriving a prefix(32) cap from a prefix(16) where 16 bits match, but others do not', async function () {
        const procAName = "SysCallTestProcRegister";
        const procAContract = Valid.SysCallTestProcRegister;
        const procACaps = [
            new beakerlib.WriteCap(0x8000,2),
            new beakerlib.RegisterCap(0, ""),
            new ThisCap(16,"ab"),
        ];

        const procBName = "Adder";
        const procBContract = Valid.Adder;
        const procBCaps = [
            new ThisCap(32,"abcd"),
            new ThisCap(32,"abxy"),
        ];

        const shouldSucceed = true;
        await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
    });
    it('Should fail when deriving a prefix(32) cap from a prefix(16) where addresses are different', async function () {
        const procAName = "SysCallTestProcRegister";
        const procAContract = Valid.SysCallTestProcRegister;
        const procACaps = [
            new beakerlib.WriteCap(0x8000,2),
            new beakerlib.RegisterCap(0, ""),
            new ThisCap(16,"ab"),
        ];

        const procBName = "Adder";
        const procBContract = Valid.Adder;
        const procBCaps = [
            new ThisCap(32,"axcd"),
        ];

        const shouldSucceed = false;
        await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
    });
    it('Should fail when deriving a prefix(16) cap from a prefix(32) where addresses are the same', async function () {
        const procAName = "SysCallTestProcRegister";
        const procAContract = Valid.SysCallTestProcRegister;
        const procACaps = [
            new beakerlib.WriteCap(0x8000,2),
            new beakerlib.RegisterCap(0, ""),
            new ThisCap(32,"abcd"),
        ];

        const procBName = "Adder";
        const procBContract = Valid.Adder;
        const procBCaps = [
            new ThisCap(16,"abcd"),
        ];

        const shouldSucceed = false;
        await stdTest(procAName, procAContract, procACaps, procBName, procBContract, procBCaps, shouldSucceed);
    });
}

// A test which uses procA to register procB. procACaps are the capabilities
// that procA is originally registered with procBCaps are the caps that it will
// attempt to register procB with.
async function stdTest(procAName, procAContract, procACaps,
                       procBName, procBContract, procBCaps, shouldSucceed) {
    const kernel = await Kernel.new();
    const functionSpec = "B(bytes24,address,uint256[])";

    const procedures1Raw = await kernel.listProcedures.call();
    const procedures1 = procedures1Raw.map(web3.toAscii)
        .map(s => s.replace(/\0.*$/, ''));
    assert(procedures1.length == 0,
        "The kernel should initially have no procedures");
    const [regEPTX, setEPTX] = await testutils.installEntryProc(kernel);

    const procedures2Raw = await kernel.listProcedures.call();
    const procedures2 = procedures2Raw.map(web3.toAscii)
        .map(s => s.replace(/\0.*$/, ''));
    // Check that the entry procedure was correctly installed.
    assert(procedures2.includes("EntryProcedure"),
        "The kernel should have an entry procedure registered");
    let entryProcedureNameRaw = await kernel.getEntryProcedure.call();
    let entryProcedureName = web3.toAscii(web3.toHex(entryProcedureNameRaw))
        .replace(/\0.*$/, '');
    assert.strictEqual(entryProcedureName,
        "EntryProcedure", "The entry procedure should be correctly set");

    const deployedContractA = await testutils.deployedTrimmed(procAContract);
    // This is the procedure that will do the registering
    // this currently requires Any because it uses logging for testing
    const tx1 = await kernel.registerAnyProcedure(procAName,
        deployedContractA.address, beakerlib.Cap.toInput(procACaps));
    // Check that procA was correctly installed.
    const procedures3Raw = await kernel.listProcedures.call();
    const procedures3 = procedures3Raw.map(web3.toAscii)
        .map(s => s.replace(/\0.*$/, ''));
    assert(procedures3.includes(procAName), "ProcA should be registered");

    {
        // Test that procA returns the correct testNum
        const functionSelectorHash = web3.sha3("testNum()").slice(2,10);
        const inputData = web3.fromAscii(procAName.padEnd(24,"\0"))
            + functionSelectorHash;
        const tx3 = await kernel.sendTransaction({data: inputData});
        const valueXRaw = await web3.eth.call({to: kernel.address,
            data: inputData});
        const valueX = web3.toBigNumber(valueXRaw);
        // we execute a test function to ensure the procedure is
        // functioning properly
        assert.equal(valueX.toNumber(), 392,
            "should receive the correct test number");
    }

    let mainTX;
    // This is the procedure that will be registered
    const deployedContractB = await testutils.deployedTrimmed(procBContract);
    {
        const functionSelectorHash = web3.sha3(functionSpec).slice(2,10);
        const encodedCapsVals = beakerlib.Cap.toInput(procBCaps).map(x=>web3.toHex(x).slice(2).padStart(64,0));
        const manualInputData
            // the name of the procedure to call (24 bytes)
            = web3.fromAscii(procAName.padEnd(24,"\0"))
            // the function selector hash (4 bytes)
            + functionSelectorHash
            // the name argument for register (32 bytes)
            + web3.fromAscii(procBName.padEnd(24,"\0")).slice(2).padEnd(32*2,0)
            // the address argument for register (32 bytes)
            + deployedContractB.address.slice(2).padStart(32*2,0)
            // the offset for the start of caps data (32 bytes)
            + web3.toHex(96).slice(2).padStart(32*2,0)
            // the caps data, which starts with the length
            // + web3.toHex(0).slice(2).padStart(32*2,0)
            + web3.toHex(encodedCapsVals.length).slice(2).padStart(32*2,0)
            // followed by the values
            + encodedCapsVals.join("");

        // when using web3 1.0 this will be good
        // try {
        //     console.log(deployedContract.methods.B(testProcName,
        //         deployedTestContract.address,[]).data)
        // } catch (e) {
        //     console.log(e)
        // }
        const inputData = manualInputData;
        const valueXRaw = await web3.eth.call({to: kernel.address,
            data: inputData});
        mainTX = await kernel.sendTransaction({data: inputData});
        const valueX = web3.toBigNumber(valueXRaw);
        if (shouldSucceed) {
            assert.equal(valueX.toNumber(), 0,
                "should succeed with zero errcode");
        } else {
            assert(valueX.toNumber() != 0, "should fail with non-zero errcode");
        }
    }

    const procedures4Raw = await kernel.listProcedures.call();
    const procedures4 = procedures4Raw.map(web3.toAscii)
        .map(s => s.replace(/\0.*$/, ''));
    if (shouldSucceed) {
        assert(procedures4.includes(procBName),
            "The correct name should be in the procedure table");
        assert.strictEqual(procedures4.length, (procedures3.length+1),
            "The number of procedures should have increased by 1");
        // TODO: check that the capabilities are correct.
        const procTableData = await kernel.returnProcedureTable.call();
        const procTable = beakerlib.ProcedureTable.parse(procTableData);
        const procBNameEncoded = web3.fromAscii(procBName.padEnd(24,'\0'));
        const procBData = procTable.procedures[procBNameEncoded];

        assert.deepStrictEqual(
            stripCapIndexVals(beakerlib.Cap.toCLists(procBCaps)),
            stripCapIndexVals(procBData.caps),
            "The requested caps should equal resulting caps");
    } else {
        assert(!procedures4.includes(procBName),
            "The correct name should not be in the procedure table");
        assert.strictEqual(procedures4.length, procedures3.length,
            "The number of procedures should have remained the same");
    }
    return mainTX;
}


// Test hack to remove data we don't care about. The kernel stores no
// information about where a capability was derived from.
function stripCapIndexVals(capData) {
    for (const cap in capData) {
        cap.capIndex = 0;
    }
}
