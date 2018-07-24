# Beaker Whitepaper

> *Every program and every privileged user of the system should operate using
> the least amount of privilege necessary to complete the job.*
>
> — Jerome Saltzer, Communications of the ACM

## Abstract

*Small summary of the contents of the whitepaper.*

## Introduction

*This whitepaper outlines the Beaker kernel and operating system.*

## Risks, Issues, and Problems of Smart Contract Systems

*This will outline what the issues we're trying to solve are.*

Smart contracts are, by their nature, unforgiving machines that follow the their
specification to the letter. This is one of the selling points of smart
contracts, but it can also be their downfall as the machine will faithfully
execute any flaw or mistake in that code. This is a well known property of smart
contracts, and is tackled in a variety of ways. By employing lower level
primitives like multi-signature wallets or escrow, the users of smart contracts
can build a more flexible and forgiving system on top of the raw and unforgiving
Ethereum machine underneath.

Building higher level systems such as wallets and escrow are a double-edged
sword in that by providing protection against the ruthlessness of the machine,
they also increase the complexity of the system one is using. This forces users
of smart contracts to strike a balance between contracts that are simple but
inflexible or flexible but complex. With this complexity comes the risk that one
small error or vulnerability can bring down a whole system. In order to prevent
this we want some form of security and control.

**TODO:** This needs to be severely improved.

## Requirements

*This sets out what we need to secure, and what tools we need to do that. It
will conclude noting that we need a form of control or introspection into the
running systems. This will dovetail with the next section.*

## Introspection via an Operating System

*This setion outlines how we build an operating system (i.e. procedures and
syscalls) and how this gives us the control needed to implement a number of
different security systems.*

### The Necessity and Advantage of Introspection and Control

*This will outline in more detail how having some control and insight into what
is happening as the system is executing. E.g. it allows us to see what each
chunk of code is doing to the state of the system, and possibly disallow that if
necessary.*

**TODO:** Let's start with the monitoring and introspection bits.

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

### Providing Introspection and Control (System Calls)

As we covered above, a traditional operating system achieves introspection and
control by interposing itself between the processes that are running and all
other parts of the system including storage, memory, and even other processes.
Each process is given its own memory and access to a processor, but is otherwise
completely isolated. In order to do even the most fundamental thing (be in print
a character to the screen or read some stored data) it must ask the operating
system to do it on its behalf. It is only through the operating system that the
process can affect the real world. Via this mechanism, we (as the owners of the
operating system) have complete control over what that process can do.

TODO: insert diagram of system calls in a traditional OS.

These requests to the operating system that a process makes are called system
calls.

#### How We Can't Implement System Calls

*TODO:* This may not be necessary.*

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
contains no state changing opcodes except `DELEGATECALL`, and that when it does
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

This delegate call is a call back into the kernel. The kernel only want's to
accept system calls from its own processes. Thankfully Ethereum provides this
feature for us. When a procedure is initially called, it is called via
`CALLCODE`. This means that that our system, which we will call our "kernel
instance", is the current storage and event space of the running code. It also
means that the `CALLER` value, which is a global read-only value, is set to the
address of out kernel instance. When our procedure does a `DELEGATECALL`, this
address is maintained. As a consequence, whenever a kernel is executing a system
call, it is able to simply check that the `CALLER` value is equal to its own
address (it is necessary for this value to be hardcoded into the instance, which
is performed during instantiation).

With this we have a kernel that is generally only accessible from its own
procedures. It must, however, also accept some form of external transaction,
this is the only way it can be triggered to execute code. As an operating system
Beaker should have no say over what kind of transactions and programs a system
wants to execute. Beaker follows a microkernel design, where the kernel itself
should stay out of the user's code as much as possible.

![With kernel instance](media/WithKernelInstance.svg)

1. Kernel instance executes a procedure by doing delegate call to the kernel.
2. The kernel fulfils this request by doing a delegate call to the contract.
3. While processing, the contract encounters a system call and does a
   delegate call to the `CALLER` value, which is the kernel instance.
4. The kernel instance checks that itself is the original caller and if so,
   calls the kernel library for processing.

So we have established that a Beaker instance has to accept two types of
transactions:

- A system call, which will come from an executing procedure that the kernel
  itself called earlier.
- An external transaction from an address associated with a user. This is what
  will trigger whatever actions the designer of the system has chosen to
  implement.

We have covered above how system calls are secured by only accepting system
calls from the procedures owned by kernel (we will cover permissions and
capabilities later), but we must also establish a procedure for external
transactions.

In order to allow maximum user freedom, rather than try and interpret and
control a format for external messages, this is left entirely to the design of
the user. As part of the kernel's initial setup, it will have a procedure which
is designated as the "entry" procedure. This entry procedure is created (and
updated if need be) by the user, and executes as a normal procedure. When the
kernel instance receives an external transaction from another Ethereum address,
it simply forwards the message data to the entry procedure.

#### The Entry Procedure

While it is completely up to the user to specify how the entry procedure works, here are some examples for how such a procedure might be implemented.

Imagine a system that has many users. These users are people with Ethereum
addresses that participate in an organtisation that is described by our system.
When they choose, they are able to execute one of the functions of this
organisation by sending a transaction to our system. Each of these users has
different levels of authority, and therefore we need some way to be certain that
only the users that we specify can execute some of the more restricted function
of the organisation of the system.

Whenever one of these transactions reaches the kernel, it is passed directly to
user code, no questions asked. It is up to our entry procedure to decide what
should happen. Let's say our kernel receives a message which requests that the
"deleteMember" function be executed to delete a certain member from the
organisation. This message will be passed on to the entry procedure, and it is
up to the entry procedure to execute that function.

However, as our entry procedure can be programmed by us, it can run some logic
to restrict this rather dangerous function. In this example, our entry procedure
will check the user's address against a list of known administrators and
determine if they are permitted to execute this procedure. If they are not, the
entry procedure can simply reject that message and rever the transaction.

#### Auditabilty and the Principle of Least Privelege

It may seem like no great gain to implement all of this additional complexity,
when in the end we simply pass the transaction to a user defined contract that
still needs to make all the same logic decisions and is subject to the same
risks and issues as any Ethereum contracts. Where the operating system model
improves this status quo is in isolation. Just as when running Linux it would be
possible to run everything as root, so too in Beaker would it be possible to run
everything in the entry procedure and have little to no benefit. What this has
allowed us to do is to isolate the highest risk portion of our code to this
entry procedure, which ideally should be kept as small and as robust as
possible.

As everywhere in computing, the princple of least privelege applies here, and
once the entry procedure has made a determination of the level of privelege
require it should pass control to more specialised more restriced procedure that
can then implement large amounts of logic without the risk of accidentally (or
maliciously) executing some of the more dangerous functions of the system.

## Security Model

*This section builds on the operating system concepts and adds the security
functionality via capabilities.*

Above we established two critical components of Beaker: introspection via system calls, and using this to achieve the principle of least privilege.

When you create a process on Linux system, that process comes with restrictions
(usually determined by the user it is run as). Even if the code in that process
asks to do something outside of what it is permitted to do, the operating system
will refuse to service that system call, and the process will therefore not be
able to escape its restricted box.

These restrictions do not generally exist *within* programs. It is not possible
in most programming languages to import a library or module and say to the "my
program sends packets over the network, and must have permission to do so, but
this section of code coming from this library should not be able to". This is
the state of Ethereum security currently. Everything is built as a single
program, without internal security boundaries (the new `STATICCALL` opcode is on
attempt to implement this).

However, now that we have established an operating system with system calls, we now have that point of control over the contracts running on our system, and we can craft policies to allow or deny any action that interacts with the rest of the system. Beaker provides a system whereby system calls from procedures can be rejected outright based on a policy set by the owner of the system.

In our example above, perhaps the entry procedure contains only a very simple
block of logic which says that if the sender is one of the administrators, the
received message is passed to another procedure which handles administrator
actions, while all other senders are passed to another procedure which handles
general members and unknown senders. This is a good example of the principle of
least privilege. If there is a bug in the code that checks which administrators
can delete users and under what conditions, that bug can only be triggered by
administrators. All other senders are immediately siphoned off to the general
member procedure. This significantly reduces the risk for both mistakes and
malicious actions.

If there is a mistake in the general member procedure, this is now less critical
as the operating system will prevent it from interacting with the critical parts
of the system, and so actions like deleting a user are prevented by the policies
of the system.

This leaves on piece of code that has full power and could potentially do
anything if it failed: the entry procedure. Now, however, the high risk code
that has all of this power is limited to a few lines of code that simply direct
different users' messages to different parts of the system. This is a small
piece of logic that can be much more easily verified.

### Capability Based Security and Their Advantages

*This section will outline how we can use the operating system to allow, deny,
and audit anything we like. This will not tackle permission/authorisation
directly, but simply shows that whatever permission system we choose can use the
operating system to disallow certain action etc.*

Now that we have this system which can allow or deny various system calls
(thereby reducing the level of privelege and risk of different sections of code
in the system), we need some way to specify the policies that provide these
barriers. If we provide a policy mechanism that is too simple, it will not be
able to provide the necessary guarantees. If it is too complex, than it is far
less auditable and becomes almost as complex as the procedure themselves.

One driving observation is that the interactions and code of Ethereum contracts
don'y match the user-based permissions of a desktop operating system. Also,
given the "hands-off" microkernel approach, we want to give as much freedom to
the designer of the system as possible. For this reason our security model needs
to be as abstract as possible.

It is also important that the system be as resistant as possible to the many
mishaps that can befall permissions systems (see the confused deputies problem
as an exemple), and be well studied in academic literature. For these reasons in
particular we have chosen a capability-based security model for Beaker.

**TODO:** How much of an overview of capabilities do we need?

### Implementing a Capability Based Security Model

*This section will outline the capability model we have designed, and how it
uses all of the about material to enforce its model.*

**NB:** What's described here is the simplest capability model we could build.
From here we should expand it to make it more complete and featureful.

**TODO:** We need to note that our capability system is based around assigning
capabilities to procedures, which it then holds when run. This can definitely be
used to model user capabilities (users are simply routed through procedures
which hold the appropriate capabilies). It does not handle dynamic capabilities,
but the current line of thinking is that we should only include those when they
are shown to be necessary.

One of the goals that would improve security and audability of a system is that
an external party or higher level "system designer" might what some control
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

*Here we list through all the kernel objects required to implement our security model and thier corresponding capabilities.*

#### Procedure Table

*Here we descibe the procedure table as an object, and how it can be changed. We
can elaborate that procedures themselves are objects and require a capability to
be accessed.*

#### Capability Table

*Here we describe the capability table as an object, and how it can be changed.*

#### Storage

*Here we describe storage as an object, and how it can be changed.*

#### Events

*Here we describe events as an object, and how it can be changed.*

#### Gas

*Here we describe gas as an object, and how it can be changed.*

## Applications

*Some information on "usage characteristics". How do you actually use this
properly. This should be high level and not necessarily include code examples or
the like.*

### Filesystem

*Here we can provide an example of how a filesystem that uses the capability
model can provide a storage abstraction to procedures*

## Conclusion

*Summarise what we have outlined above and why it is useful.*
