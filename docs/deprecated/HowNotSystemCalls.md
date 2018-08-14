# How We Can't Implement System Calls

*Present the limitations of things like static call, and the difficulty of
coroutines.*

One newer feature of the EVM is *static call*. This performs a call with no side
effects, only return values. THis is like asking the EVM to ensure the called
contract has no capacities other than returning a value. The called contract
will revert execution if it tries to do anything else. This is a cheap way to do
static analysis, but it only allows for checking that the contract makes zero
state changes. It does not allow for a contract that performs a subset of
permitted state changes.

One way to overcome this limitation is to return to the calling contract and let
it handle the restricted operation. The contract would simply return the
information telling our operating system what changes it wanted to effect.
However, if we want to execute a number of state changes during the execution of
our contrat there is no practial way to make those state changes via the kernel
and then return to the execution of our contract.

It is possible implement the necessary bookkeeping in contracts, but as there
are no interrupts or concurrency on the EVM, and each contract has its own
memory space, this would be an expensive and prohibitive solution.

Once you return to the calling contract, all memory is lost, so that memory must
be saved somewhere.

It cannot be saved in storage, as during a static call we do not have access to
storage.

If it is returned it could be arbitrarily large, and it would require to set
aside the correct return buffer for storing this memory before the call, which
cannot be known.

The only remaining alternative is to "re-run" all the opcodes and simply skip
the system calls we have already used. This could result in enormous execution
times if there are many system calls.

Another possibility is to execute the procedure only once, and mark out all the
system calls that need to be performed in the final return value. Note that this
also means that the size of the return value must be known ahead-of-time, so the
contract must state upfront its return size. This means the number of syscalls
cannot be arbitrary. This is not an acceptable trade-off for a general system.