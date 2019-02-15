# Procedure Execution

## Format
### Procedure
```rust
struct Procedure {
    id: u8,
    address: u256,
    capability_list: &CList
}
```
A procedure contains: an `id`, which defines it's position in the procedure table, it's contract `address` and it's `capability_list`, where its capabilities are assigned by the kernel.

### Capability List
```rust
struct CList {
    cap_type: Map<CapabilityType, Vec<Capability>>,
}
```

Each capability list contains a seperate list of capabilities for each possible capability type.

#### Capability Types
Each capability type is referenced by a byte identifier. In this case, from 0 to 10.
```rust
enum CapabilityType {
    /// Null Capability
    NullCap             = 0,
    /// Add capability to procedure with given identifier
    ProcedurePushCap    = 1,
    /// Delete capability from procedure with given identifier and index
    ProcedureDeleteCap  = 2,
    /// Call procedure by given id and arguments.
    ProcedureCall       = 3,
    /// Register procedure with given identifier.
    ProcedureRegister   = 4
    /// Delete procedure by identifier.
    ProcedureDelete     = 5,
    /// Set the procedure with given identifier as the entry procedure.
    ProcedureEntry      = 6,
    /// Read from the memory by the given address.
    StorageRead         = 7,
    /// Write to the memory by the given address.
    StorageWrite        = 8,
    /// Append log record with given topics.
    LogWrite            = 9,
    /// Send gas to an external address
    GasSend             = 10,
    /// The total amount of gas received from user.
    GasRecieved         = 11,
}
```

### Syscall

When invoking a capability we use the `invoke` system call. The signature of invoke is as follows:
```rust
fn invoke(cap_type: CapabilityType, cap_index: u8, input: &Vec<u256>, output: &mut Vec<u256>)
```
Where `cap_type` is the type of the capability, `cap_index` is the index of the capability, `input` is the memory location and size for reading the input parameters and `output` is the output memory location for the kernel to write the result. When the kernel reads the `input` it will parse it according to the `cap_type`.

## Example: Storage Capability
Here we describe how a procedure will write an arbitrary, 32-byte value to address 0x07. This should be executed via a capability attached to a procedure. This will be the basis from which we extend to other capabilities.

### Format
``` rust
struct StorageCap {
    location: u256,
    size: u256
}

struct StorageCapRead(StorageCap);
struct StorageCapWrite(StorageCap);

```

A storage capability contains a `location` of 32 bytes, and `size` that designates the amount of 32 byte keys available starting from `location`. There are two types of storage capabilities: read and write, each with a different parameter format and output format.

```rust

/// Here we define an abstract interface when invoking a capability type.
trait Invoke {
    type Input: From<Vec<u256>>;
    type Output: From<Vec<u256>>;
    type Error;

    fn invoke(&self, input: &Input, output: &Output);
}

/// Write Params
struct StorageCapWriteParam {
    keys: Vec<u256>,
    values: Vec<u256>
}

/// Write Output
struct StorageCapWriteOutput {
    prev_values: Vec<u256>
}

/// Read Params
struct StorageCapReadParam {
    keys: Vec<u256>
}

/// Read Output
struct StorageCapReadOutput {
    values: Vec<u256>
}

```

When reading, the parameters are parsed as a list of u256 keys, and for each key a value is returned. When writing, we parse two lists of keys and values to be written. Output in this case are the old values being written over.

#### Summary of Message Format

The format of the memory message is as follows: 1 byte for the capability type, one byte for the capability index. Each procedure has a list of capabilities, and the capability index is an index into this list. (**TODO**: Why do we need to specify the capability type here? Isn't that contained within the capability?). After that is the start address where we will write, as well as the number of addresses we will write to.

* Byte 0: Capability Type
* Byte 1: Capability Index
* Bytes 2 to 34: Number of addresses to be written to (this values is referred to as `n`)
* Bytes 34 to 34+n\*3`: Storage keys to be written to.
* Bytes 34+n\*3 to 34+2\*n\*32: Values to be written into each corresponding key.

For example, to write the value `0x34` into the storage key `0x7` the message format will be as follows.

* Byte 0: `0x7`
* Byte 1: A single byte corresponding to the index of this capability in the list.
* Bytes 2-33: `0x1`, we are writing to a single address.
* Bytes 34-65: `0x7`, the single address we are writing to.
* Bytes 66-97: `0x34`, the single value we are writing into to.

#### Summary of the Capability Format

The write capability is 2 32-byte values. A storage address and an offset/size. The capability allows writing to any value with `address + offset`.

* Byte 0: The capability type (for write it is `0x7`)
* Bytes 1-32: `address`, the start address.
* Bytes 33-64: `offset`, the number of addresses (starting with `address` that can be written)

For example, the ability to write to storage location `0x7` only is:

1. `0x07`
2. `0x07`
3. `0x01`

That is: storage location `0x7`, a single address. Fully padded (with dashes to separate values) this is:

```
0x07-0000000000000000000000000000000000000000000000000000000000000007-0000000000000000000000000000000000000000000000000000000000000001
```

### Execution

In the case where we wish to write a `0x01` into storage location `0x70`, we need to have a storage write capability in the `StorageWrite` capability list (index 7), that has either a location `0x70` or a location with a size that overlaps `0x70`. In this case we assume we have a storage write capability at index 0, with a `location`: 0x70, and `size`: 1.

```rust

// Assuming we have a StorageCapWrite at index 0;
let store_cap_write = 0;
let input = StorageWriteParam { keys: vec![0x70], values: vec![0x01]};
let mut output = StorageWriteOutput { values: Vec<u256> };

// Invoke System Call
invoke(CapabilityType::StorageWrite, store_cap, &input, &mut output);

// Read Output, first value should be old value
assert!(output.values[0] == 0u256);

```










