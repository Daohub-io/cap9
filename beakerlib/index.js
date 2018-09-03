exports.parseProcedureTable = parseProcedureTable;
function parseProcedureTable(val) {
    const procTable = {};
    for (let i = 0; i < val.length;) {
        const proc = {};
        // Key
        proc.key = web3.toHex(val[i]); i++;
        if (proc.key == "0x0") break;
        // KeyIndex
        proc.keyIndex = web3.toHex(val[i]); i++;
        // Location
        proc.location = web3.toHex(val[i]); i++;
        // Capabilities
        proc.caps = [];
        const nCaps = val[i].toNumber(); i++;
        for (let j = 0; j < nCaps; j++) {
            const cap = {};
            const length = web3.toHex(val[i]); i++;
            cap.type = web3.toHex(val[i]); i++;
            // (length - 1) as the first value is the length
            cap.values = [];
            for (let k = 0; k < (length-1); k++) {
                cap.values.push(web3.toHex(val[i])); i++;
            }
            proc.caps.push(cap);
        }
        procTable[proc.key] = proc;
    }
    return procTable;
}

exports.printProcedureTable = printProcedureTable;
function printProcedureTable(procTable) {
    for (const procKey of Object.keys(procTable)) {
        const proc = procTable[procKey];
        // Print key
        console.log(`Key: ${proc.key}`);
        // Print keyIndex
        console.log(`  KeyIndex: ${proc.keyIndex}`);
        // Print location
        console.log(`  Location: ${proc.location}`);
        // Print Capabilities
        console.log(`  Capabilities(${proc.caps.length} keys)`);
        for (const i in proc.caps) {
            const cap = proc.caps[i];
            console.log(`    Capability[${i}]: Type: ${cap.type}`);
            for (const j in cap.values) {
                console.log(`      ${j}: ${cap.values[j]}`)

            }
        }
    }
}

class Cap {
    constructor(type) {
        this.type = type;
    }
    toIntegerArray() {
        const keyValArray = this.keyValues();
        // The plus one is to account for the type value
        const headerArray = Array.from([keyValArray.length+1, this.type]);
        return headerArray.concat(keyValArray);
    }
    static toInput(caps) {
        let input = new Array();
        for (const cap of caps) {
            input = input.concat(cap.toIntegerArray());
        }
        return input;
    }
}

class WriteCap extends Cap {
    constructor(address, size) {
        super(0x7);
        this.address = address;
        this.size = size;
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        return Array.from([this.address,this.size]);
    }
}
exports.WriteCap = WriteCap;
exports.Cap = Cap;
