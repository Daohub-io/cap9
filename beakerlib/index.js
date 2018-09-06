class ProcedureTable {
    constructor(procTable) {
        this.procedures = procTable;
    }
    // get procedures() {
    //     return this.procTable;
    // }
    static parse(val) {
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
        return new ProcedureTable(procTable);
    }
    static stringify(procTable) {
        let str = "";
        for (const procKey of Object.keys(procTable.procedures)) {
            const proc = procTable.procedures[procKey];
            // Print key
            str += `Key: ${proc.key}\n`;
            // Print keyIndex
            str += `  KeyIndex: ${proc.keyIndex}\n`;
            // Print location
            str += `  Location: ${proc.location}\n`;
            // Print Capabilities
            str += `  Capabilities(${proc.caps.length} keys)\n`;
            for (const i in proc.caps) {
                const cap = proc.caps[i];
                str += `    Capability[${i}]: Type: ${cap.type}\n`;
                for (const j in cap.values) {
                    str += `      ${j}: ${cap.values[j]}\n`;
                }
            }
        }
        return str;
    }
}
exports.ProcedureTable = ProcedureTable;


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
exports.Cap = Cap;

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

// Currently the log is just a boolean flag determining whether it can log.
class LogCap extends Cap {
    constructor(topics) {
        super(0x9);
        this.topics = topics;
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        return Array.from([this.topics.length].concat(this.topics));
    }
}
exports.LogCap = LogCap;
