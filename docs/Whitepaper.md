# Beaker Whitepaper


## Abstract
Small summary of the contents of the whitepaper

## Introduction

*This whitepaper outlines the Beaker kernel and operating system.*

## Risks, Issues, and Problems of Smart Contract Systems

*This will outline what the issues we're trying to solve are.*

## Requirements

*This sets out what we need to secure, and what tools we need to do that.  It
will conclude noting that we need a form of control or introspection into the
running systems. This will dovetail with the next section.*

*This will outline in more detail how having some control and insight into what
is happening as the system is executing. E.g. it allows us to see what each
chunk of code is doing to the state of the system, and possibly disallow that if
necessary.*

We have established that there is a requirement for enacting security measures
in smart contract systems. There are two distinct ways of thinking about this
that have very specific implications. There is the "bottom-up" approach, where
each of the components of a system is secured (for some definition of secured)
so that when it is used by the system, the system can be confident that the
component will operate within certain bounds. This approach often involves
things such as static verification.

The alternative approach is the "top-down" approach, where it is the system that
imposes restrictions on its components. For example when a system executes a
stored program it restricts what that stored program can do. This requires some
level of control over the program.

If you imagine an early computer that simply executed a sequence of instructions
and manipulated hardware devices, the bottom-up approach would be to verify that
your program is correct, and then run the program. Once the program is running,
you have no control.

With the advent of operating systems, this relationship changed. Now when a
program ran, it wasn't just excuted blindly by the machine. When it encountered
an instruction that required hardware access, it suspended the running program
and passed that instruction to the operating system, which then decided what to
do. This is more similar to the top-down approach in that the system had full
control over what was and wasn't allowed, at the cost of some run-time
monitoring.

On Ethereum the situation is very similar. Smart contracts are thoroughly
audited and tested, but once they are on the blockchain they execute on the raw
machine. If we want a system where we can interpose ourselves between our
components and potentially dangerous resources (such as our storage) we can make
stronger guarantees about the safety of our system, without having to apply the
same level of verification to every component.

So, how do we interpose ourself between potentially dangerous contracts and our
system? The same way we always have in computer systems. We create an operating
system, and require that all contracts interact with critical resources only
through system calls. Once a contract is only operating through system calls,
the operating system has the final say on what the contract can do.

## Kernel

This setion outlines how we build an operating system kernel (i.e. procedures and
syscalls) and how this gives us the control needed to implement a number of
different security systems.

### Procedures

Here we outline procedures, how they would be created, deployed and how they need a system call interface to be able to interact with the kernel.

### System Calls

This will outline how system calls work.

#### How We Can't Implement System Calls

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

#### How We Can Implement System Calls

*Present the solution using delegate call and on chain code verification. It
would be nicer if on chain code verification had more prominence, but this makes
the most sense when reading the whitepaper.*

We can avoid the problem of moving memory around with static call by using
delegate call. A naive attempt at fixing the problems with static call is to
call directly into the kernel and ask the kernel to do the work. This way, when
the kernel has completed the processing, it returns and the contract can
continue processing with its memory and program counter intact. However, as our
contract is running under the static flag, when we call the kernel again, the
kernel will also be under the static flag and will not be able to complete the
system calls, as it is also unable to effect stateful changes.

In order to allow the kernel to make stateful changes we would need to delegate
call (or similar) into the kernel. As delegate call is potentially effectful, it
is also not available under the static flag, so we would need to delegate call
into the contract too. However, this means we lose all of the security benefits
of static call! In order to get those security guarantees back (i.e. that no
stateful changes will occur under the contract) we must implement them
ourselves. Thankfully, because we are using system calls for all of our state
changing work, this verification is quite simple. A contract should contain no
state changing opcodes, except those used to execute system calls.

The sequence of EVM opcodes below execute a system call. The input and output
parameters of the system call are set prior to these instructions, and are the
responsibility of the designer of the contract (presumably with significant
assistance from libraries). These instructions simple ensure that the system
call is only calling to the original kernel instance and nothing more. Because
this sequence of instructions is the only sequence of instructions with a state
changing opcode (`DELEGATECALL`) it is simple to verify on-chain that a contract
contains no state changing opcodes except `DELEGATECALL, and that when it does
it is only in this form (i.e. system call form).

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

1. Kernel calls a contract using delegate call.
2. Contract calls back to kernel with delegate call.

In this setup the caller value is always the kernel, so the kernel can be sure
it is only being called by itself when doing the system calls.

![With kernel instance](media/WithKernelInstance.svg)

1. Kernel instance executes a procedure by doing delegate call to the kernel.
2. The kernel fulfils this request by doing a delegate call to the contract.
3. While processing, the contract encounters a system call and does a
   delegate call to the `CALLER` value, which is the kernel instance.
4. The kernel instance checks that itself is the original caller and if so,
   calls the kernel library for processing.

## Security Model

*This section builds on the operating system concepts and adds the security
functionality via capabilities.*

### Capability Based Security and Their Advantages

*This section will outline how we can use the operating system to allow, deny,
and audit anything we like. This will not tackle permission/authorisation
directly, but simply shows that whatever permission system we choose can use the
operating system to disallow certain action etc.*

### Implementing a Capability Based Security Model

*This section will outline the capability model we have designed, and how it
uses all of the about material to enforce its model.*

**NB:** What's described here is the simplest capability model we could build.
From here we should expand it to make it more complete and featureful.

One of the goals that would improve security and audability of a system is that
and external party or higher level "system designer" might what some control
over what the various conracts in th system can do. This would allow them to
compartmentalise areas of code and ensure that code only has the priveleges it
requires, focussing attention on more critical high-risk code. Even if another
member of the organisation updates a contract under his or her control, the
system designer should be able to limit the potential damage of an error or
malign action by sandboxing that contract.

This can be done by giving each contract (a *procedure* in Beaker parlance) a
set of capabilities, outside of which it cannot act. Even if the procedure is
updated, unless the developer also has the right to increase its capabilities,
system designers and auditors have some guarantees that it will not do something
harmful to parts of the system outside its purview.

The simplest model is simply to give every procedure a list of permitted
actions. Procedure creation is in two steps:

- Creation/Update - Where the contract bytecode is uploaded to the kernel.
- Permission assignation - Where somebody with the appropriate authorisation
  sets the capabilities of the procedure.

**TODO:** Include a diagram of this.

It is critical to note that the capability system proposed here does not attempt
to deal at all with *"users"*. If a particular system hs users (which is to be
expected) it is left to the creators of that system to dictate how that is
organised and implemented. By default, Beaker routes all external transactions
through a (modifiable) procedure which acts as a form of "gatekeeper". It is
within this procedure that decisions about what each user can do are made.

When a procedure is created, it has zero capabilities available to it in its
list. If, for example, it needs to modify the storage value at `0x7`, it will
need to be provided with that permission by a separate permission assignation.
In this workflow, the procedure is deployed by a developer, and the permissions
are assigned by the system designer once he approves this. The workflow around
how permissions are requested and designed are left to the system creators.

**SIDENOTE:** Perhaps we allow a procedure to be run on every syscall what does
whichever system checks the designers deem appropriate although the large number
of procedure calls make this very expensive. It is always possible to provide
system creators with such hooks.

In this situation it would be important to give the system designer powerful
design tools.

**What can't this do?** Something like storage locations or procedure ids can't
be chosen dynamically, as delegation of capabilities does not occur.

Everything is statically determined by the system designer (although permissions
may be changed at any point).

This design has advantages over more dynamic, flexible, capability systems, as
it explains, in a very static and assessable manner:

1. Where do permissions come from?
2. How are they set?
3. How are they enforced?

### Kernel Objects and Capabilities
Here we list through all the kernel objects required to implement our security model and thier corresponding capabilities

#### Procedure Table
Here we descibe the procedure table as an object, and how it can be changed.
We can elaborate that procedures themselves are objects and require a capability to be accessed.

#### Capability Table
Here we describe the capability table as an object, and how it can be changed.

#### Storage
Here we describe storage as an object, and how it can be changed.

#### Events
Here we describe events as an object, and how it can be changed.

#### Gas
Here we describe gas as an object, and how it can be changed.

## Applications

*Some information on "usage characteristics". How do you actually use this
properly. This should be high level and not necessarily include code examples or
the like.*

### Filesystem
Here we can provide an example of how a filesystem that uses the capability model can provide a storage abstraction to procedures

## Conclusion

*Summarise what we have outlined above and why it is useful.*
