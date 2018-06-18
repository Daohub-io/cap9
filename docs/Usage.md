# Usage

**NB:** This currently has some conflicts with SystemCallDesign.md.

This document is to present an overview of what it is like to interact with
Beaker's capability system.

## Introduction

Currently the capability system is designed around procedure-based capabilities.
That is, users do not possess capabilities, only procedures do. It is then up to
the system designer to implement procedures that handle the authentication and
permissions of users. For example, and expected design pattern (and one we will
support with the basic templates provided with Beaker) is a procedure which
handles all incoming user transactions, and maintains a permission list for all
user's of the system. This procedure is completely custom, but one feature it
will have is the ability to execute any other procedure. When a user sends a
transaction to the kernel instance requesting one of the procedures be executed,
the transaction will first be processed by this "reception procedure". If the
reception procedure determines that a user has sufficient privileges, it will
execute the procedure on the user's behalf.

## The Caller

But how will the code for this reception procedure look? The following is
pseudo-code to approximate the features we expect to be available in Beaker.
Each process (i.e. current execution of a procedure) has a CList, which holds
the process' current capabilities. This is similiar to a file handle table in a
regular operating system, each capability is simply an index into this table. We
happen to know that at index \#2 (in reality we will use variable names, not
indices) of the CList is the capability to execute any procedure, let's call it
`execAny`. In our capability system there is no such system call as "exec", we
only have `invoke` (analagous to seL4's `send`), which will invoke a capability.

In order to execute a particular procedure (let's give it the procedure id
`0xad`) we need to create something we can invoke. `execAny` doesn't have enough
information, it doesn't know which procedure to execute. In reality there will
simply be a function `exec(procId, input)`, but that hides what's going on under
the hood, we want to see everything in terms of the `invoke` calls. We want
something of the form `invoke(cap,input)`, where `cap` is the capability (or
index into the CList) and `input` is simply and user specified data we wish to
pass along. Note that the `input` should not have any capabilitiy or permission
information we would have to check, the `input` should be completely transparent
to the capability system, and it should be able to simply pass it along. If it
contained permission-sensitive data it would need checking and complicate the
capability system, defeating the benefits.

So, given this, it seems that `cap` needs to contain all the designation
information, including what we want to do with it (be it, read, write, execute
or the like). In order to make our `execAll` capability into something we can
`invoke` we need to provide it with the `procId` of the procedure we want to
execute. For this we need another (currently pseudo) function called `mint()`.
This will take our old capability and create a new one that we can invoke
directly. In the listing below we will take our `execAll` capability (at index
\#2) and narrow the scope of it to something like `exec-0ad`. We can know invoke
that capability as it has all the information we need. In this case mint is also
executed by `invoke` but that is under the hood. At the bytecode level, `invoke`
is simply a delegate call to the kernel. `mint` will append the new capability
to the running process' capability list, and return the index so we can use it
later.

```c
1: let cap = mint(#2,0xad)
2: let ret = invoke(cap,"data")
```

This `exec` capability is expected to be the most prevalent, as we will be doing
little direct "hardware" access (like read and write) ourselves. These will be
handles by whichever procedure is responsible. These capabilities must exist at
some level however. For example, reading and writing of storage locations. Here the capabilities are written out as text, but in reality they will be indices into the CList.

```c
invoke("writeAt0x8000","somedata")
let out = invoke("readAt0x8000")
assert(eq(out,"somedata"))
```

What if we want to give a child process that we call particular capabilities?
This is known as delegation in a capability system and is important. Let's say
(for example) we want to call a public library to modify one of our storage
locations. We want to specify which storage location, and give it the ability to
do so. We must therefore pass a write capability to that storage location when
we execute it. These capabilities must be sent via the input data of `invoke`,
and it must be something the receiver is asking for, however, we must also
explicitly tell the kernel that we are doing this, otherwise the receiver could
fake it and access our capabilities. We need some form of delegation. This
delegation is only relevant during `exec`. It is up to the code (via a library,
not the user) to correctly format the input data of `exec` to contain the
capabilities we want to pass the callee. The input field of an `exec` capability invocation will be as follows:

```
exec
  caps(3):
    cap1
    cap2
    cap3
  various-input-data-that-is-interpreted-by-the-callee
```

When the `exec` invocation is received by the kernel, `cap1`, `cap2`, and `cap3`
are copied to the CList of the callee. But how does the callee know how to call
these 3 capabilities? Perhaps the callee will receive those as well, and
procedures in BeakerOS will have 2 kinds of arguments.

## The Callee

So know we know how to call some code, how do we go about being called? Beaker
itself places no limitations on how a callee interprets input data, except that
it will specially format and pass on capabilities. Every `exec` invocation can
pass on as many capabilities as it chooses, and the start of the input data will
contain the capabilities for the callee to use. These capabilities need to be in
this special argument format, as they are new indices, and are provided by the
kernel. Therefore, every procedure needs to know how to interpret capability
arguments, even if it wishes to simply skip them and continue to the regular
input data. Currently, every capability is simply a `uint256` into the CList.
The first `uint256` of the input data is the number of capabilites being sent
(it may be zero). The next `n` values (where n is the value of the first
`uint256`) will be the capabilities, and everything after that will be the input
data, formatted as the user chooses.

Taking the example above, the callee will receive the following:

```
0x3 (32 bytes)
  cap1Index (32 bytes)
  cap2Index (32 bytes)
  cap3Index (32 bytes)
various-input-data-that-is-interpreted-by-the-callee
```

It is up to the callee to interpret the meaning of the capabilities and their
order. For example, consider a procedure that takes two storage locations and
copies the value of the first into the second, and then wipes the value of the
first. Obviously as these storage locations must be read from and written to,
they must be passed as capbilities. The first will be `rw` (read/write) as it
will need to both read from and write to that location, the second will just be
`write` as it does not need to read from that location. Let's assume that the caller has the capability `readWriteAny` at index \#3.

```c
let s1 = 0xee
let s2 = 0xff
let cap1 = mint(#3,0xee) // cap1 is now readWriteAt0xee
let cap2 = mint(#3,0xff) // cap1 is now readWriteAt0xff
let execCap = mint(#2,0xad) // the capability to execute our procedure
let out = invoke(execCap,cap1,cap2,"") // we send no other data
```

The caller will can then interpret this:

```c
// take our source and destination from the list of capability arguments
let [from,to]  = capabilityArguments
// the regular arguments are empty
assert(eq(capabiltyArguments,""))
// we can now preform our algorithm
// first we need a read capability from our first capability
let source = mint(from,"read")
// perform the read
let sourceData = invoke(source,"")
// next we need a write capability from our first capability
let destination = mint(to,"write")
// perform the write
invoke(destination,sourceData)
// then we want to zero-out our source, so now we need a write capability
let writeSource = mint(from,"write")
// last, we invoke this to zero out our source storage address
invoke(writeSource,"")
```

## Kernel Services

TODO: this needs a lot of discussion and fleshing out.

Why a "write" procedure and no direct write access? There is of course a "direct
write" capability, but it is expected to be rare for procedures to be writing
directly to storage addresses themselves, they will likely want different types
of tables, or a file system for example (this is influenced by a micro-kernel
approach, seL4 takes a similar approach).
