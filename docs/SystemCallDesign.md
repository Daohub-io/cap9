# System Calls

**NB:** This does not include the implementation of system calls, although tries
to be aware of there requirements in design.

## Necessary System Calls

First we identify which opcodes we need to use via system calls. These are all
the "state-changing" opcodes.

- SSTORE
- LOG0-4
- SELFDESTRUCT
- CREATE
- CREATE2
- CALL
- CALLCODE
- DELEGATECALL

Second we identify non-state-changing opcodes we may also want to control.

- SLOAD

Thirdly we make this into system calls. Here are the first three fundamental
system calls:

- Read
- Write
- LOG0-4

The rest will come.

System calls are executed in a procedure using DELEGATECALL, with the input
buffer including all the information about the system call. A system call, as
seen in the contract bytecode will be something like this:

```
0: PUSH out_size
1: PUSH out_offset
2: PUSH in_size
3: PUSH in_offset
4: CALLER
5: GAS
6: DELEGATECALL
```

There are a number of vairables here, so let's go through them. First, let's
reorder this like a function call to make things clearer (i.e. useing JULIA
syntax).

```
delegatecall(gas, caller, in_offset, in_size, out_offset, out_size)
```

The first argument is the maximum amount of gas we will allow this system call
to use. Here we are using the `GAS` opcode to find out how much gas we have
left. This way we are telling the system call it can use all the gas we have
left. Out-of-gas errors in relation to procedure calls and system calls will be
presented elsewhere, but usually we give system calls full trust as it
dramatically simplifies things.

The second argument is the address we will call. When we see this system call we
are in a procedure. This means we are in a contract that has been called via
`DELEGATECALL` from a kernel instance, meaning the value of `CALLER` is the
address of the kernel instance. As the kernel instance is the contract to which
we want to send the system call, this is the address we give to `DELEGATECALL`.
This doesn't need to be checked, as these sequence of instructions can only call
to the `CALLER`.

The next 4 parameters are the input and output parameters. `in_offset` and
`out_offset` are the locations in memory where the input and output values of a
system call are stored. These are completely at the discretion of the procedure
code, as they are highly variable and have no impact on the system call itself.

`in_size` and `out_size` are also mostly relevant to the calling procedure, but
are highly dependent on the format of the system call. These values will of
course need to match the requirements of the system call, but carry little to no
risk to the system call, provided the input decoder of the kernel is secure.

## System Call Format

As each system call is an opaque `DELEGATECALL`, in order to determine what a
system call does we need to pass both the type of system call, and the necessary
parameters to that system call via the input data of the call. We will also need
to standardise the return value (output) from each system call so that
procedures can reliably handle the result.

Let's create an initial (potentially naive) draft of the Write system call.
These system call is analagous to `SSTORE` and stores a single 32-byte value at
an address (more efficient array stores and the like are potential future
feature). The input data needs to provide the following information:

1. This is a Write system call.
2. The storage address to which we want to write (e.g. 0x11).
3. The value which we want to store there.

It might be valueable to take inspiration from the ABI currently used by
Solidity et al. to influence the design of this format to try and reduce
impedence mismatch, but it is likely to be substantially different, so we
shouldn't restrict ourselves to trying to adhere to it.

Side Note: I might take that back, the ABI is far too high level, we care
nothing of types. Current system calls only support "stack level" parameters,
and I believe it should stay that way. If we need to pass more complex data we
will define that when necessary. Right now we only need to know how to put
32-byte values (i.e. stack cells) into the input data.

The first thing that needs to be tackled is the defining which system call it
is. There are two ways we might tackle this "opcode style" and "function style".
Opcode style is where each system call is an integer, just like an opcode, and
how most efficient systems deal with system calls. Function style is where we
have something longer and name-based, like the actual name of the system call or
it's hash. The advantage of opcode style is that it is more well-defined and a
simpler mechanism, and does not involve parsing and all the potential system
failures and exploits that come with it (**Jake:** I'm heavily in favour of the
opcode style). The following will assume an opcode style.

For this document we will assume that Write uses the system call index `0x02`
(`0x00` being reserved for something else and `0x01` being Read). The system
call indentifier will consist of 3 bytes. The first byte is a version number,
which will be defined as `0x00` but is included to allow for future changes.
This is important because this is work in progress and we don't want to risk
losing control of the system calls (although hopefully it will not need to be
bumped). The next two bytes is the identifier for the system call. We are
unlikely to need more than 255 system calls, therefore the second byte of the
three will likely always be zero (it is easier to remove this later than to add
it). Where we discuss system calls in this document we will often just refer to
the third byte, which actually chooses which system call to use.

So, the first three bytes are therefore the system call designator. The bytes
after that are the input. Write takes two input parameters: the location (which
will be a capability in future) and the value. Both are 32-byte values. It's
possible to greatly reduce the size of 32-byte values when they are small
numbers, but for now we will for 2 32-byte values.

Therefore a Write system call's input data that stores the value `0x1234` at location `0x7` will look like this:

```

0x000002 - 0x0000000000000000000000000000000000000000000000000000000000000007 - 0x0000000000000000000000000000000000000000000000000000000000001234
```

This is 3+32+32=67 bytes.

For output, this is entirely dependent on the system call. For lower level
errors such as out-of-gas, invalid opcode, and the like, a return code will be
left on the stack by `DELEGATECALL` which needs to be appropriately handled by
the procedure. Higher level errors returned by the kernel will need handled via
the return data of the system call. For example, perhaps the Write call is to an
area of storage that is not permitted and the kernel rejects it. For Write calls
the return data can be simply. Either it succeeds, or it fails for some reason.
A single return byte is enough to signify success or failure, and even with 254
possible error values in order to return more information to the calling
procedure. Therefore the return value from the Write call is a single byte, with
zero representing success and non-zero representing failure of some kind (with
error codes yet to be defined).

# DEPRECATED

**NB:** These are some ideas on how to call to different places, but ideally we
can keep it as simple as above.

This is a "raw" system call that provides the functionality we need, but it is
also insecure. We only want this contract to be calling back to the kernel. Even
when we genuinely want to call another external contract we need to do that via
the kernel so that the kernel can perform the necessary security checks. In
order to make sure we are only calling to the kernel, we add the following
security checks:

```
0: PUSH out_size
1: PUSH out_offset
2: PUSH in_size
3: PUSH in_offset
4: PUSH kernel_instance_address
5: DUP1
6: CALLER
7: EQ
8: NOT
9: PUSH err_address
10: JUMPI
11: GAS
12: DELEGATECALL
```

Here we have added instructions 5 through 10. These instructions check that the
address we are calling to is the current caller. In all cases the current caller
is the kernel instance, so this ensures that system calls can only occur back to
the instance this contract has been called from.