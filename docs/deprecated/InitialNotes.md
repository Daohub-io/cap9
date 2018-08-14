# The Design of the Permission System

There are two different principles we have discussed:

- Securing and limiting the actions of contracts.
- Defining a permission system to control what subordinate contracts can do.

The first might not be called a permission system, and is what I would consider
"bottom-up" security. In bottom-up security, each contract has a static set of
actions which it can take. These actions are a static property of the bytecode,
and must be accepted and rejected in their entirety.

[insert diagram]

In this example, MyContract is calling LibContract using DelegateCall.
LibContract contains some code that stores a value at storage location 0x5. This
can be seen via static analysis of the code. Maybe this code won’t be executed,
but we cannot be sure. We therefore say that LibContract has the "capacity" to
modify the storage at 0x5.

When we design MyContract to call LibContract we need to be aware that this call
can arbitrarily change the value at 0x5. It also means that if another contract
performs DelegateCall to MyContract this call can also arbitrarily change the
value at 0x5, as MyContract might call LibContract at any time.

This "bottom-up" approach is limited by the power of our static analysis. For
example, if LibContract takes a storage address as part of its input, it can
store an arbitrary address. If we want to restrict this arbitrary storage
access, we must modify the code of LibContract to do so.

One way we might do this is by adding range checks before each store call to
ensure that stores are restricted to a range we accept. We have already
implemented this, and there are numerous was to do it.

One disadvantage this has is that there is more code in more places. If we place
the "checking" code in each contract, there are more instances to check and bugs
are harder to patch.

One newer feature of the EVM is StaticCall. This performs a call with no side
effects, only return values. THis is like asking the EVM to ensure the called
contract (like LibContract in the example above) has no capacities other than
returning a value. The called contract will revert execution if it tries to do
anything else. This is a cheap way to do static analysis, but it only allows for
checking for zero capacities, not a custom set.

One way to get past this limitation is to return to the calling contract and let
it handle the restricted operation. The disadvantage of this system is that
there is significantly more bookkeeping involved, as the current state and
opcode position must be saved and resumed. THis is problematic as there are no
interrupts on the EVM, and each contract has its own memory space.

If we use arbitrary system calls (i.e. multiple calls of different types) in a
contract, the main issue becomes how do you "resume" a contract.

Once you return to the calling contract, all memory is lost, so that memory must
be saved somewhere.

It cannot be saved in storage, as during a StaticCall we do not have access to
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
cannot be arbitrary.

Any system attempting to combine StaticCall with effectfyuleffectful opcodes
(such as SSTORE) must go down this route.

[insert listing]

For this "tail syscall" method to work we would need to store an array of
syscalls (in memory) that the contract executes. However, as the return buffer
must be specified ahead-of-time, we must use a fixed array size. This means we
must know ahead-of-time what the maximum number of syscalls will be. This is
another thing that developers would have to manage (just like stack size, except
a little harder).

**NB:** None of this covers capabilities and passing of those tokens as it relies on
system calls, which we haven’t yet established.

The proposed resumption method involves memory copying in the issue on github,
but this also requires knowing the memory size ahead-of-time.

Also, how expensive is copying memory back and forth. This could be more
expensive, although does allow for a more consistent experience.

Let’s assume that the issue with memory copying is resolved. How do we manage
the security assertions we’re trying to solve?

If we return to the "bottom-up" conversation from earlier, we know the "upper
bounds" of what a contract can do, and naturally when a contract calls another
contract it absorbs all those capacities. Therefore, we shouldn’t link a new
contract unless we are happy with its security metadata. But what if we want to
dynamically modify the permissions of the contracts we’re calling? Perhaps there
is an existing library contract (using the Beaker system) and we want to reduce
its permissions, or perhaps we want to give third party contract different
permissions depending on who is calling it.

For example: let’s say we have an organisation with many members. Each of these
members record a statement which is stored in the organisation. Each member may
update their statement whenever they please, however, it must not contain swear
words. A new contract is deployed to handle these update and enforce the
restriction.

None of the above is impacted by our system, however, we also need to make sure
that members can only modify their own statements. In order to accomplish this
we might modify this contract to only allow modifications to each statement from
the correct address. In this case we need to update the modification contract to
do this authorisation.

Now what if we want to update this to include statements from groups of members?
Now the modification is becoming more complex. What if each member also has
voting rights? Now the authentication needs to be duplicated, and managed in
both locations. Alternatively we might move authentication into another
contract. Now we have created a permission system that needs to control
permissions more generally.

One way to do this is a capability-based permission system. This would allow
each member to give access to the statement modification contract to modify
their contract, while the contract itself can remain "dumb" to the world of
permissions and authorisation. This permission system also allows users to
restrict which permissions they give so that the statement modification system
cannot modify a user's votes in the voting system.

Such a system relies on system calls as it is the only way that capabilities can
be checked. In most cases it does allow StaticCall as system calls and
capabilities are just messages for the kernel to read and implement.

A capability is a passable token which functions as:

- A designation (for example the storage location 0x5).
- An authorisation (for example read and write).

The designation component is trivial, but the authorisation part is more
complex. Contracts must be built to receive these capabilities as input, and
these capabilities must make some reference to the kernel memory in order to be
unforgeable.

**Note:** Capabilities must be tied to a particular execution, not contract or even
transaction (the capabilities must be held in kernel memory). Account
capabilities need to be held in storage.

## DelegateCall and System Calls

We can avoid the problem of moving memory around by using DelegateCall. A naive
attempt at fixing the problems with StaticCall is to call directly into the
kernel and ask the kernel to do the work. This way, when the kernel has
completed the processing, it returns and the contract can continue processing
with its memory and program counter intact. However, as our contract is running
under the static flag, when we call the kernel again, it will also be under the
static flag and will not be able to complete the system calls, as it is also
unable to effect stateful changes.

We would need to DelegateCall into the kernel. As effectful DelegateCall is not
available under the static flag, we must DelegateCall into the contract too.
However, this means we lose all of the security benefits of StaticCall! In order
to get those security guarantees back (i.e. that no stateful changes will occur
under the contract) we must implement them ourselves. Thankfully, because we are
using system calls for all of our state changing work, all we need to prove is
that our contract has no state changing code, except for the DelegateCalls into
our kernel (in order to support multiple kernels we will also need to support
dynamic kernel addresses).

Dynamic kernel addresses (argument order of DelegateCall not respected here):

```
CALLER
DUP
EQ
NOT
PUSH erraddr
JUMPI
DELEGATECALL
```

The next challenge is the calling back into the kernel, which is also an attack
vector. Thankfully capabilities help with this, but an extra layer of protection
is advisable. One of the primary security features is to only accept system
calls from the procedure (contract) it is currently returning.

![Without kernel instance](media/WithoutKernelInstance.svg)

1. Kernel calls a contract using DelegateCall.
2. Contract calls back to kernel with DelegateCall.

In this setup the caller value is always the kernel, so the kernel can be sure
it is only being called by itself when doing the system calls.

![With kernel instance](media/WithKernelInstance.svg)

1. Kernel instance executes a procedure by doing DelegateCall to the kernel.
2. The kernel fulfils this request by doing a DelegateCall to the contract.
3. While processing, the contract encounters a system call and does a
   DelegateCall to the `CALLER` value, which is the kernel instance.
4. The kernel instance checks that itself is the original caller and if so,
   calls the kernel library for processing.
