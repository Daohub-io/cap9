
const CAP_TYPE = {
    NULL                 : 0,
    PROC_CAP_PUSH        : 1,
    PROC_CAP_DELETE      : 2,
    PROC_CALL            : 3,
    PROC_REGISTER        : 4,
    PROC_DELETE          : 5,
    PROC_ENTRY           : 6,
    STORE_WRITE          : 7,
    LOG                  : 8,
    GAS_SEND             : 9
}

exports.CAP_TYPE = CAP_TYPE;

function stringToKey(name) {
    return web3.fromAscii(name.padEnd(24,'\0'));
}
exports.stringToKey = stringToKey;

class ProcedureTable {
    constructor(procTable) {
        this.procedures = procTable;
    }
    byName(name) {
        return this.procedures[stringToKey(name)];
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
                for (let k = 0; k < (length - 1); k++) {
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
        const headerArray = Array.from([keyValArray.length + 1, this.type]);
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
        super(CAP_TYPE.STORE_WRITE);
        this.address = address;
        this.size = size;
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        return Array.from([this.address, this.size]);
    }
}
exports.WriteCap = WriteCap;

class LogCap extends Cap {
    constructor(topics) {
        super(CAP_TYPE.LOG);
        this.topics = topics;
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        return Array.from(this.topics);
    }
}
exports.LogCap = LogCap;

// Currently the call is a list of procedure keys it can call. If the list is
// empty it means that any procedure can be called.
class CallCap extends Cap {
    // keys should be a list of strings
    constructor(keys) {
        super(CAP_TYPE.PROC_CALL);
        if (!keys) {
            this.keys = [];
        } else {
            this.keys = keys;
        }
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        const val = Array.from(this.keys.map(x => web3.fromAscii(x.padEnd(32, '\0'))));
        return val;
    }
}
exports.CallCap = CallCap;

class RegisterCap extends Cap {
    // A RegisterCap is just a boolean value, a procedure can or cannot
    // register new procedures
    constructor() {
        super(CAP_TYPE.PROC_REGISTER);
        this.keys = [];
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        const val = Array.from(this.keys.map(x => web3.fromAscii(x.padEnd(32, '\0'))));
        return val;
    }
}
exports.RegisterCap = RegisterCap;

class DeleteCap extends Cap {
    // A DeleteCap is just a boolean value, a procedure can or cannot
    // register new procedures
    constructor() {
        super(CAP_TYPE.PROC_DELETE);
        this.keys = [];
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        const val = Array.from(this.keys.map(x => web3.fromAscii(x.padEnd(32, '\0'))));
        return val;
    }
}
exports.DeleteCap = DeleteCap;

class SetEntryCap extends Cap {
    // A DeleteCap is just a boolean value, a procedure can or cannot
    // register new procedures
    constructor() {
        super(CAP_TYPE.PROC_ENTRY);
        this.keys = [];
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        const val = Array.from(this.keys.map(x => web3.fromAscii(x.padEnd(32, '\0'))));
        return val;
    }
}
exports.SetEntryCap = SetEntryCap;

class PushCap extends Cap {
    // A PushCap is just a boolean value that indicates whether a procedure
    // can push caps
    constructor() {
        super(CAP_TYPE.PROC_CAP_PUSH);
        this.keys = [];
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        const val = Array.from(this.keys.map(x => web3.fromAscii(x.padEnd(32, '\0'))));
        return val;
    }
}
exports.PushCap = PushCap;

class CapDeleteCap extends Cap {
    // A PushCap is just a boolean value that indicates whether a procedure
    // can push caps
    constructor() {
        super(CAP_TYPE.PROC_CAP_DELETE);
        this.keys = [];
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        const val = Array.from(this.keys.map(x => web3.fromAscii(x.padEnd(32, '\0'))));
        return val;
    }
}
exports.CapDeleteCap = CapDeleteCap;

exports.SysCallResponse = {
    SUCCESS: 0,
    READFAILURE: 11,
    WRITEFAILURE: 22,
    LOGFAILURE: 33,
    CALLFAILURE: 44,
}
