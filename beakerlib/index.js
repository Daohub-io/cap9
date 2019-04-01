
const CAP_TYPE = {
    PROC_CALL            : 3,
    PROC_REGISTER        : 4,
    PROC_DELETE          : 5,
    PROC_ENTRY           : 6,
    STORE_WRITE          : 7,
    LOG                  : 8,
    ACC_CALL             : 9
}

exports.CAP_TYPE = CAP_TYPE;

function capTypeToSize(capType) {
    if (capType == PROC_CALL) {
        return 1;
    } else if (capType == STORE_WRITE) {
        return 2;
    } else if (capType == LOG) {
        return 5;
    } else if (capType == PROC_REGISTER) {
        return 1;
    } else if (capType == PROC_DELETE) {
        return 1;
    } else if (capType == PROC_ENTRY) {
        return 0;
    } else if (capType == ACC_CALL) {
        return 1;
    } else {
        throw new Error("invalid capability type");
    }
}

class ProcedureTable {
    constructor(procTable) {
        this.procedures = procTable;
    }
    // get procedures() {
    //     return this.procTable;
    // }
    static parse(val) {
        const procTable = {};
        const nVals = val[0];
        for (let i = 1; i < nVals;) {
            const proc = {};
            // Key
            proc.key = web3.toHex(val[i]); i++;
            // Location
            proc.location = web3.toHex(val[i]); i++;
            // KeyIndex
            proc.keyIndex = web3.toHex(val[i]); i++;
            // Capabilities
            proc.rawcaps = [];
            const nCaps = val[i].toNumber(); i++;
            // Here, j represents the cap index
            for (let j = 0; j < nCaps; j++) {
                const cap = {};
                const capSize = val[i].toNumber(); i++;
                const capType = val[i].toNumber(); i++;
                cap.type = capType;
                cap.values = [];
                for (let k = 0; k < (capSize-2); k++) {
                    cap.values.push(web3.toHex(val[i])); i++;
                }
                proc.rawcaps.push(cap);
            }
            proc.caps = [];
            proc.caps[CAP_TYPE.PROC_CALL] = [];
            proc.caps[CAP_TYPE.PROC_REGISTER] = [];
            proc.caps[CAP_TYPE.PROC_DELETE] = [];
            proc.caps[CAP_TYPE.PROC_ENTRY] = [];
            proc.caps[CAP_TYPE.STORE_WRITE] = [];
            proc.caps[CAP_TYPE.LOG] = [];
            proc.caps[CAP_TYPE.ACC_CALL] = [];
            for (const rawcap of proc.rawcaps) {
                switch (rawcap.type) {
                    case CAP_TYPE.PROC_CALL:
                        proc.caps[rawcap.type].push(CallCap.fromKeyValues(rawcap.values));
                        break;
                    case CAP_TYPE.PROC_REGISTER:
                        break;
                    case CAP_TYPE.PROC_DELETE:
                        break;
                    case CAP_TYPE.PROC_ENTRY:
                        break;
                    case CAP_TYPE.STORE_WRITE:
                        proc.caps[rawcap.type].push(WriteCap.fromKeyValues(rawcap.values));
                        break;
                    case CAP_TYPE.LOG:
                        proc.caps[rawcap.type].push(LogCap.fromKeyValues(rawcap.values));
                        break;
                    case CAP_TYPE.ACC_CALL:
                        proc.caps[rawcap.type].push(AccCallCap.fromKeyValues(rawcap.values));
                        break;
                    default:
                        console.log(rawcap)
                        throw new Error(`unknown captype ${rawcap.type}: ${rawcap}`);
                }
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
        const headerArray = Array.from([keyValArray.length + 3, this.type, 0]);
        return headerArray.concat(keyValArray);
    }
    // Convert clists based on types.
    static toCLists(caps) {
        const cLists = [];
        cLists[CAP_TYPE.PROC_CALL] = [];
        cLists[CAP_TYPE.PROC_REGISTER] = [];
        cLists[CAP_TYPE.PROC_DELETE] = [];
        cLists[CAP_TYPE.PROC_ENTRY] = [];
        cLists[CAP_TYPE.STORE_WRITE] = [];
        cLists[CAP_TYPE.LOG] = [];
        cLists[CAP_TYPE.ACC_CALL] = [];
        for (const cap of caps) {
            cLists[cap.type].push(cap);
        }
        return cLists;
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
        if ((typeof address === "string") || (typeof address === "number")) {
            this.address = web3.toBigNumber(address);
        } else {
            this.address = address;
        }
        if ((typeof size === "string") || (typeof size === "number")) {
            this.size = web3.toBigNumber(size);
        } else {
            this.size = size;
        }
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        return Array.from([this.address, this.size]);
    }
    static fromKeyValues(values) {
        if (values.length != 2) {
            throw new Error("Incorrect number of values for WRITE cap");
        }
        const baseAddress = values[0];
        const additionalKeys = values[1];
        return new WriteCap(web3.toBigNumber(baseAddress), web3.toBigNumber(additionalKeys));
    }
}
exports.WriteCap = WriteCap;

class LogCap extends Cap {
    constructor(topics) {
        super(CAP_TYPE.LOG);
        if (topics.length > 4) {
            throw new Error("too many topics");
        }
        // Input topics must be hex strings or bignums. If it is a string it is
        // interpreted as bytes, that is, left-aligned in 32-bytes. If it is
        // longer than 32 bytes throw an error.
        this.topics = [];
        for (const topic of topics) {
            if (typeof topic === "string") {
                if (!topic.startsWith("0x")) {
                    throw new Error("Topic must be prefixed with \"0x\"");
                }
                if (topic.length > 66) {
                    throw new Error("Topic must be 32 bytes or less");
                }
                this.topics.push(web3.toBigNumber(topic.padEnd(66,0)));
            } else {
                throw new Error("Topic must be a hex-string");
            }
        }
        // this.topics = topics;
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        const topic1 = (this.topics.length >= 1) ? this.topics[0] : 0;
        const topic2 = (this.topics.length >= 2) ? this.topics[1] : 0;
        const topic3 = (this.topics.length >= 3) ? this.topics[2] : 0;
        const topic4 = (this.topics.length >= 4) ? this.topics[3] : 0;
        return Array.from([this.topics.length, topic1, topic2, topic3, topic4]);
    }
    static fromKeyValues(values) {
        if (values.length != 5) {
            throw new Error("Incorrect number of values for LOG cap");
        }
        const nTopics = values[0];
        if (nTopics > 4 || nTopics < 0) {
            throw new Error("Invalid number of topics");
        }
        const topics = [];
        for (let i = 0; i < nTopics; i++) {
            topics.push(values[i+1]);
         }
        return new LogCap(topics);
    }
}
exports.LogCap = LogCap;

class CallCap extends Cap {
    // keys should be a list of strings
    constructor(prefixLength, baseKey) {
        super(CAP_TYPE.PROC_CALL);
        if (baseKey.length > 24) {
            throw new Error(`key too long ${baseKey}`);
        }
        this.baseKey = baseKey.padEnd(24,'\0');
        this.prefixLength = prefixLength;
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        // The baseKey will take up the last 24 bytes
        // baseKey24 is the given key correctly padded to 24 bytes, left aligned
        const baseKey24 = web3.fromAscii(this.baseKey.padEnd(24, '\0'))
        // baseKeyHex is baseKey24, hex-encoded, and is therefore 48 chars. The
        // "0x" is removed from the start of the string.
        const baseKeyHex = web3.toHex(baseKey24).slice(2);
        // prefixHex is the prefix length hex-encoded and padded to two chars (a
        // single byte). The "0x" is removed here also.
        const prefixHex = web3.toHex(this.prefixLength).slice(2).padStart(2,'0');
        // There are 7 bytes between the prefix length and the start of the base
        // key.
        const undefinedFill = web3.toHex("".padEnd(7,'\0')).slice(2);
        // We string these together in the correct order.
        const key = "0x" + prefixHex + undefinedFill + baseKeyHex;
        const val = Array.from([key]);
        return val;
    }
    static fromKeyValues(values) {
        if (values.length != 1) {
            throw new Error("Incorrect number of values for CALL cap");
        }
        const val = "0x"+values[0].slice(2).padStart(64,0);
        const prefixHex = val.slice(0,4);
        const baseKeyHex = "0x"+val.slice(18);
        return new CallCap(web3.toBigNumber(prefixHex).toNumber(), web3.toAscii(baseKeyHex));
    }
}
exports.CallCap = CallCap;

class RegisterCap extends Cap {
    // A RegisterCap is just a boolean value, a procedure can or cannot
    // register new procedures
    constructor(prefixLength, baseKey) {
        super(CAP_TYPE.PROC_REGISTER);
        if (baseKey.length > 24) {
            throw new Error("key too long");
        }
        this.baseKey = baseKey;
        this.prefixLength = prefixLength;
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        // The baseKey will take up the last 24 bytes
        // baseKey24 is the given key correctly padded to 24 bytes, left aligned
        const baseKey24 = web3.fromAscii(this.baseKey.padEnd(24, '\0'))
        // baseKeyHex is baseKey24, hex-encoded, and is therefore 48 chars. The
        // "0x" is removed from the start of the string.
        const baseKeyHex = web3.toHex(baseKey24).slice(2);
        // prefixHex is the prefix length hex-encoded and padded to two chars (a
        // single byte). The "0x" is removed here also.
        const prefixHex = web3.toHex(this.prefixLength).slice(2).padStart(2,'0');
        // There are 7 bytes between the prefix length and the start of the base
        // key.
        const undefinedFill = web3.toHex("".padEnd(7,'\0')).slice(2);
        // We string these together in the correct order.
        const key = "0x" + prefixHex + undefinedFill + baseKeyHex;
        const val = Array.from([key]);
        return val;
    }
}
exports.RegisterCap = RegisterCap;

class DeleteCap extends Cap {
    // keys should be a list of strings
    constructor(prefixLength, baseKey) {
        super(CAP_TYPE.PROC_DELETE);
        if (baseKey.length > 24) {
            throw new Error("key too long");
        }
        this.baseKey = baseKey;
        this.prefixLength = prefixLength;
    }
    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        // The baseKey will take up the last 24 bytes
        // baseKey24 is the given key correctly padded to 24 bytes, left aligned
        const baseKey24 = web3.fromAscii(this.baseKey.padEnd(24, '\0'))
        // baseKeyHex is baseKey24, hex-encoded, and is therefore 48 chars. The
        // "0x" is removed from the start of the string.
        const baseKeyHex = web3.toHex(baseKey24).slice(2);
        // prefixHex is the prefix length hex-encoded and padded to two chars (a
        // single byte). The "0x" is removed here also.
        const prefixHex = web3.toHex(this.prefixLength).slice(2).padStart(2,'0');
        // There are 7 bytes between the prefix length and the start of the base
        // key.
        const undefinedFill = web3.toHex("".padEnd(7,'\0')).slice(2);
        // We string these together in the correct order.
        const key = "0x" + prefixHex + undefinedFill + baseKeyHex;
        const val = Array.from([key]);
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

class AccCallCap extends Cap {
    constructor(callAny, sendValue, ethAddress) {
        super(CAP_TYPE.ACC_CALL);
        this.callAny = callAny;
        this.sendValue = sendValue;
        this.ethAddress = ethAddress;
    }

    // Format the capability values into the values that will be stored in the
    // kernel. Must be defined for all subclasses
    keyValues() {
        const value = new Uint8Array(32);
        const callAny = this.callAny ? 0b10000000 : 0;
        const sendValue = this.sendValue ? 0b01000000 : 0;
        value[0] = callAny | sendValue;
        if (!this.ethAddress) {
            value.fill(0,12,32);
        } else {
            const byteArray = hexToByteArray(this.ethAddress);
            value.set(byteArray, 32 - byteArray.length);
        }
        const val = Array.from(["0x"+bufferToHex(value)]);
        return val;
    }
}
exports.AccCallCap = AccCallCap;

exports.SysCallResponse = {
    SUCCESS: 0,
    READFAILURE: 11,
    WRITEFAILURE: 22,
    LOGFAILURE: 33,
    CALLFAILURE: 44,
}

// Convert a Uint8Array to a hex string (non-prepended)
function bufferToHex(buffer) {
    return Array
        .from (new Uint8Array (buffer))
        .map (b => b.toString (16).padStart (2, "0"))
        .join ("");
}


function hexToByteArray(string) {
    const sa = string.startsWith("0x") ? string.slice(2) : string;
    const s = ((sa.length % 2) == 0) ? sa : "0"+sa;
    const r = [];
    let i = 0;
    while ((i+1) < s.length) {
        const ins = "0x"+s[i]+s[i+1];
        r.push(parseInt(ins))
        i += 2;
    }
    return r;
}
